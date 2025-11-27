// This file contains the MoveRuntime wrapper implementation.
// It utilizes MoveVM and InMemoryStorage for executing functions and publishing modules.

use anyhow::Result;
use move_binary_format::file_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::{ModuleId, TypeTag};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas::UnmeteredGasMeter;


use crate::move_vm_state::MoveVMState;

/// Simple runtime wrapper around `move-vm` for executing functions and publishing modules.
pub struct MoveRuntime {
	vm: MoveVM,
	storage: InMemoryStorage,
	state: MoveVMState,
}

impl MoveRuntime {
	/// Open the runtime using the default persistent DB path (see README).
	pub fn new() -> Result<Self> {
		let state = MoveVMState::open_default()?;
		let mut storage = InMemoryStorage::new();
		state.load_into_storage(&mut storage)?;
		// For simplicity we initialise the VM with no custom natives.
		let vm = MoveVM::new(vec![]).map_err(|e| anyhow::anyhow!(format!("VM init error: {:?}", e)))?;
		Ok(MoveRuntime { vm, storage, state })
	}

	/// Publish a module (bytes) with the given sender address.
	/// The module is applied to the in-memory storage and persisted to RocksDB.
	pub fn publish_module(&mut self, module_bytes: Vec<u8>, sender: AccountAddress) -> Result<()> {
		let storage_clone = self.storage.clone();
		let mut session = self.vm.new_session(storage_clone);
		let mut gas = UnmeteredGasMeter;

		session
			.publish_module(module_bytes.clone(), sender, &mut gas)
			.map_err(|e| anyhow::anyhow!(format!("publish error: {:?}", e)))?;

		let (res, new_storage) = session.finish();
		let (changeset, _events) = res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

		let mut storage = new_storage;
		storage
			.apply(changeset)
			.map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

		// update our runtime storage
		self.storage = storage.clone();

		// persist module bytes to DB so they are available on next startup
		let compiled = CompiledModule::deserialize_with_defaults(&module_bytes)
			.map_err(|e| anyhow::anyhow!(format!("deserialize error: {:?}", e)))?;
		let module_id = compiled.self_id();
		self.state.save_module(&module_id, &module_bytes)?;

		Ok(())
	}

	/// Publish a bundle of modules atomically. This helps resolving inter-module dependencies.
	pub fn publish_module_bundle(&mut self, modules: Vec<Vec<u8>>, sender: AccountAddress) -> Result<()> {
		let storage_clone = self.storage.clone();
		let mut session = self.vm.new_session(storage_clone);
		let mut gas = UnmeteredGasMeter;

		session
			.publish_module_bundle(modules.clone(), sender, &mut gas)
			.map_err(|e| anyhow::anyhow!(format!("publish bundle error: {:?}", e)))?;

		let (res, new_storage) = session.finish();
		let (changeset, _events) = res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

		let mut storage = new_storage;
		storage
			.apply(changeset)
			.map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

		// update runtime storage
		self.storage = storage.clone();

		// persist each compiled module to DB
		for module_bytes in modules.into_iter() {
			let compiled = CompiledModule::deserialize_with_defaults(&module_bytes)
				.map_err(|e| anyhow::anyhow!(format!("deserialize error: {:?}", e)))?;
			let module_id = compiled.self_id();
			self.state.save_module(&module_id, &module_bytes)?;
		}

		Ok(())
	}

	/// Attempt to publish modules in an order that satisfies dependencies by retrying
	/// individual publishes. Each module is published with its declared `self_id().address()` as sender.
	pub fn publish_modules_ordered(&mut self, modules: Vec<Vec<u8>>) -> Result<()> {
		use std::collections::VecDeque;
		let mut queue: VecDeque<Vec<u8>> = VecDeque::from(modules);
		let mut made_progress = true;
		let mut last_err: Option<anyhow::Error> = None;

		while !queue.is_empty() && made_progress {
			made_progress = false;
			let len = queue.len();
			for _ in 0..len {
				let bytes = queue.pop_front().unwrap();
				// try to deserialize to get module address
				match CompiledModule::deserialize_with_defaults(&bytes) {
					Ok(compiled) => {
						let mod_id = compiled.self_id();
						let sender = AccountAddress::from_hex_literal(&format!("0x{}", mod_id.address().short_str_lossless())).unwrap_or(mod_id.address().clone());
						let res = self.publish_module(bytes.clone(), sender);
						match res {
							Ok(()) => made_progress = true,
							Err(e) => {
								last_err = Some(e);
								// push back for another attempt later
								queue.push_back(bytes);
							}
						}
					}
					Err(e) => {
						last_err = Some(anyhow::anyhow!(format!("deserialize error: {:?}", e)));
						// cannot determine sender, give up on this module
					}
				}
			}
		}

		if !queue.is_empty() {
			return Err(last_err.unwrap_or_else(|| anyhow::anyhow!("failed to publish modules due to unresolved dependencies")));
		}
		Ok(())
	}

	/// Execute an entry function. `type_args` are Move `TypeTag`s and `args` are serialized
	/// arguments as Vec<u8> (Move simple-serialized values).
	pub fn execute_entry_function(
		&mut self,
		module_id: &ModuleId,
		function_name: &str,
		type_args: Vec<TypeTag>,
		args: Vec<Vec<u8>>,
	) -> Result<()> {
		let storage_clone = self.storage.clone();
		let mut session = self.vm.new_session(storage_clone);
		let mut gas = UnmeteredGasMeter;

		// convert type tags to VM runtime types
		let mut ty_args_loaded = vec![];
		for tag in type_args.iter() {
			let ty = session
				.load_type(tag)
				.map_err(|e| anyhow::anyhow!(format!("load type error: {:?}", e)))?;
			ty_args_loaded.push(ty);
		}

		let ident = IdentStr::new(function_name).map_err(|e| anyhow::anyhow!(e.to_string()))?;

		session
			.execute_entry_function(module_id, ident, ty_args_loaded, args, &mut gas)
			.map_err(|e| anyhow::anyhow!(format!("exec error: {:?}", e)))?;

		let (res, new_storage) = session.finish();
		let (changeset, _events) = res.map_err(|e| anyhow::anyhow!(format!("finish error: {:?}", e)))?;

		let mut storage = new_storage;
		storage
			.apply(changeset)
			.map_err(|e| anyhow::anyhow!(format!("apply error: {:?}", e)))?;

		self.storage = storage;
		Ok(())
	}
}

