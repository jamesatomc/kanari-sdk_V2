// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use super::reroot_path;
use anyhow::{bail, Context, Result};
use clap::*;
use kanari_crypto::wallet::load_wallet;
use kanari_types::address::Address;
use move_package::BuildConfig;
use std::path::PathBuf;

/// Publish the Move module to the blockchain
#[derive(Parser)]
#[clap(name = "publish")]
pub struct Publish {
    /// Path to the Move package (defaults to current directory)
    #[clap(long = "package-path")]
    pub package_path: Option<PathBuf>,

    /// Gas limit for the transaction
    #[clap(long = "gas-limit", default_value = "1000000")]
    pub gas_limit: u64,

    /// Gas price in Mist
    #[clap(long = "gas-price", default_value = "1000")]
    pub gas_price: u64,

    /// Account address publishing the module (from wallet)
    #[clap(long = "sender")]
    pub sender: String,

    /// Wallet password (required for signing)
    #[clap(long = "password")]
    pub password: Option<String>,

    /// Skip signature (for testing only)
    #[clap(long = "skip-signature")]
    pub skip_signature: bool,

    /// RPC endpoint
    #[clap(long = "rpc", default_value = "http://127.0.0.1:3000")]
    pub rpc_endpoint: String,
}

impl Publish {
    pub fn execute(self, path: Option<PathBuf>, config: BuildConfig) -> Result<()> {
        let rerooted_path = reroot_path(path.or(self.package_path.clone()))?;

        // Validate sender address
        let _sender_addr = Address::from_hex(&self.sender)
            .with_context(|| format!("Invalid sender address: {}", self.sender))?;

        println!("📦 Building Move package...");

        // Build the package
        let compiled_package = config.compile_package(&rerooted_path, &mut std::io::stderr())?;

        println!("✅ Package compiled successfully!");
        println!("   Modules: {}", compiled_package.all_modules().count());

        // Get compiled modules
        let modules: Vec<_> = compiled_package.all_modules().collect();

        if modules.is_empty() {
            bail!("No modules found in package");
        }

        // Load wallet if not skipping signature
        let wallet = if !self.skip_signature {
            let password = self
                .password
                .as_ref()
                .context("Password required for signing (use --password or --skip-signature)")?;

            let w = load_wallet(&self.sender, password).context(
                "Failed to load wallet. Make sure the wallet exists and password is correct",
            )?;

            println!(
                "🔐 Wallet loaded: {} (curve: {})",
                self.sender, w.curve_type
            );
            Some(w)
        } else {
            println!("⚠️  Test mode: Skipping signature");
            None
        };

        println!("\n📤 Publishing modules to blockchain...");
        println!("   RPC: {}", self.rpc_endpoint);

        for module_unit in &modules {
            let module = &module_unit.unit.module;
            let module_name = module.self_id().name().to_string();
            let module_bytecode = {
                let mut bytes = vec![];
                module.serialize(&mut bytes)?;
                bytes
            };

            println!("\n  📝 Module: {}", module_name);
            println!("     Size: {} bytes", module_bytecode.len());
            println!("     Address: {}", module.self_id().address());
            println!("     Functions: {}", module.function_defs.len());

            // Estimate gas
            let estimated_gas = 60_000 + (module_bytecode.len() as u64 * 10);
            let estimated_cost = estimated_gas * self.gas_price;
            println!("     Estimated Gas: {} units", estimated_gas);
            println!(
                "     Estimated Cost: {} Mist ({:.6} KANARI)",
                estimated_cost,
                estimated_cost as f64 / 1_000_000_000.0
            );

            if estimated_gas > self.gas_limit {
                eprintln!(
                    "     ⚠️  Warning: Estimated gas ({}) exceeds limit ({})",
                    estimated_gas, self.gas_limit
                );
            }

            // Create and sign transaction
            if let Some(ref wallet) = wallet {
                println!(
                    "     🔑 Signing transaction with {} key...",
                    wallet.curve_type
                );

                // In production, this would:
                // 1. Create proper transaction with module bytecode
                // 2. Sign with wallet private key
                // 3. Broadcast to RPC endpoint
                // 4. Wait for confirmation

                println!("     ⚠️  Not yet implemented: Blockchain submission");
            }
        }

        println!("\n✅ Package build and validation complete!");
        println!("⚠️  Note: Blockchain submission not yet implemented");
        println!("   Use `execute_with_engine()` method for direct engine access");

        Ok(())
    }
}
