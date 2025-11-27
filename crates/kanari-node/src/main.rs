use anyhow::Result;
use hex::encode as hex_encode;
use kanari_move_runtime::{MoveRuntime};
use kanari_crypto::wallet::list_wallet_files;
use kanari_types::framework_path::FrameworkPath;

use move_core_types::account_address::AccountAddress;

use std::path::PathBuf;
use std::{env, thread, time::{Duration, SystemTime, UNIX_EPOCH}};

fn main() -> Result<()> {
	// Simple CLI: subcommands: run | publish-all | list-wallets | publish-file <path>
	let args: Vec<String> = env::args().collect();
	let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("run");

	match cmd {
		"list-wallets" => {
			let wallets = list_wallet_files()?;
			for (addr, selected) in wallets {
				println!("{}{}", addr, if selected { " (selected)" } else { "" });
			}
			return Ok(());
		}

		"publish-file" => {
			let path = match args.get(2) {
				Some(p) => PathBuf::from(p),
				None => {
					eprintln!("Usage: publish-file <path-to-bytecode.mv>");
					std::process::exit(2);
				}
			};

			let mut rt = MoveRuntime::new()?;
			let bytes = std::fs::read(&path)?;
			// use system address as sender
			let sender = AccountAddress::from_hex_literal("0x2")?;
			println!("Publishing {}...", path.display());
			rt.publish_module(bytes, sender)?;
			println!("Published.");
			return Ok(());
		}

		"inspect" => {
			let path = match args.get(2) {
				Some(p) => PathBuf::from(p),
				None => {
					eprintln!("Usage: inspect <path-to-bytecode.mv>");
					std::process::exit(2);
				}
			};
			let bytes = std::fs::read(&path)?;
			match move_binary_format::file_format::CompiledModule::deserialize_with_defaults(&bytes) {
				Ok(compiled) => {
					println!("ModuleId address: {}", compiled.self_id().address());
					println!("ModuleId name: {}", compiled.self_id().name());
				}
				Err(e) => eprintln!("Failed to deserialize module: {:?}", e),
			}
			return Ok(());
		}

		"publish-all" => {
			// Verify framework paths exist
			FrameworkPath::verify_paths()?;

			let modules_dir = FrameworkPath::kanari_system_modules();
			let mut rt = MoveRuntime::new()?;
			let sender = AccountAddress::from_hex_literal("0x2")?;

			// First publish stdlib dependencies if present
			if let Some(deps_dir) = FrameworkPath::find_stdlib_modules() {
				println!("Found stdlib at: {}", deps_dir.display());
				
				let stdlib_files = FrameworkPath::get_module_files(&deps_dir)?;
				if !stdlib_files.is_empty() {
					let dep_modules = FrameworkPath::read_modules(&stdlib_files)?;
					
					for path in &stdlib_files {
						println!("Queued stdlib module {}", path.display());
					}

					println!("Publishing MoveStdlib dependency bundle ({} modules)...", dep_modules.len());
					let std_sender = AccountAddress::ONE;
					
					if let Err(e) = rt.publish_module_bundle(dep_modules.clone(), std_sender) {
						eprintln!("Failed to publish stdlib bundle: {:?}", e);
						println!("Falling back to ordered publish for stdlib modules...");
						if let Err(e2) = rt.publish_modules_ordered(dep_modules.clone()) {
							eprintln!("Ordered publish for stdlib also failed: {:?}", e2);
						} else {
							println!("Published MoveStdlib modules (ordered fallback).");
						}
					} else {
						println!("Published MoveStdlib bundle.");
					}
				}
			} else {
				println!("No stdlib modules found, skipping stdlib publish.");
			}

			// Now collect and publish the main package modules as a bundle
			let module_files = FrameworkPath::get_module_files(&modules_dir)?;
			if !module_files.is_empty() {
				let modules = FrameworkPath::read_modules(&module_files)?;
				
				for path in &module_files {
					let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("<file>");
					println!("Queued {} for publishing", name);
				}

				println!("Publishing main module bundle ({} modules)...", modules.len());
				if let Err(e) = rt.publish_module_bundle(modules, sender) {
					eprintln!("Failed to publish main bundle: {:?}", e);
				} else {
					println!("Published main module bundle.");
				}
			}

			println!("Publish-all complete.");
			return Ok(());
		}


		"run" => {
			// fallthrough to node run
		}
		"start" => {
			// alias for "run"
		}
		_ => {
			eprintln!("Unknown command: {}. Available: run | start | publish-all | publish-file <path> | list-wallets | inspect <path>", cmd);
			std::process::exit(2);
		}
	}


	println!("Kanari node starting...");
	let mut tick: u64 = 0;
	loop {
		tick += 1;
		let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
		let wallets = list_wallet_files().unwrap_or_default();
		println!("[{}] tick={} wallets={} uptime_seconds={}", hex_encode(&now.to_be_bytes()), tick, wallets.len(), now);
		thread::sleep(Duration::from_secs(5));
	}
}
