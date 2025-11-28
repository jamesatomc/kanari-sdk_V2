// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use clap::*;
use kanari_crypto::wallet::load_wallet;
use kanari_types::address::Address;
use move_core_types::{account_address::AccountAddress, language_storage::TypeTag, parser};

/// Call a Move function on the blockchain
#[derive(Parser)]
#[clap(name = "call")]
pub struct Call {
    /// Object ID of the package, which contains the module
    #[clap(long = "package")]
    pub package: String,

    /// The name of the module in the package
    #[clap(long = "module")]
    pub module: String,

    /// Function name in module
    #[clap(long = "function")]
    pub function: String,

    /// Type arguments to the generic function being called.
    /// All must be specified, or the call will fail.
    /// Example: 0x1::coin::KANARI
    #[clap(long = "type-args")]
    pub type_args: Vec<String>,

    /// Simplified ordered args like in the function syntax.
    /// ObjectIDs, Addresses must be hex strings.
    /// Example: 0x123 1000 true
    #[clap(long = "args")]
    pub args: Vec<String>,

    /// Sender/Caller address (from wallet)
    #[clap(long = "sender")]
    pub sender: String,

    /// Wallet password (required for signing)
    #[clap(long = "password")]
    pub password: Option<String>,

    /// Gas limit for the transaction
    #[clap(long = "gas-limit", default_value = "200000")]
    pub gas_limit: u64,

    /// Gas price in Mist
    #[clap(long = "gas-price", default_value = "1000")]
    pub gas_price: u64,

    /// Skip signature (for testing)
    #[clap(long = "skip-signature")]
    pub skip_signature: bool,

    /// RPC endpoint
    #[clap(long = "rpc", default_value = "http://localhost:9944")]
    pub rpc_endpoint: String,

    /// Dry run (estimate gas without executing)
    #[clap(long = "dry-run")]
    pub dry_run: bool,
}

impl Call {
    pub fn execute(self) -> Result<()> {
        println!("📞 Preparing function call...");

        // Validate addresses
        let _package_addr = Address::from_hex(&self.package)
            .with_context(|| format!("Invalid package address: {}", self.package))?;
        let _sender_addr = Address::from_hex(&self.sender)
            .with_context(|| format!("Invalid sender address: {}", self.sender))?;

        println!("\n📋 Call Details:");
        println!("   Package: {}", self.package);
        println!("   Module: {}", self.module);
        println!("   Function: {}", self.function);
        println!("   Sender: {}", self.sender);
        println!("   Gas Limit: {}", self.gas_limit);
        println!("   Gas Price: {}", self.gas_price);

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
                "   🔐 Wallet loaded: {} (curve: {})",
                self.sender, w.curve_type
            );
            Some(w)
        } else {
            println!("   ⚠️  Test mode: Skipping signature");
            None
        };

        // Parse type arguments
        let _type_args = if !self.type_args.is_empty() {
            let mut parsed = Vec::new();
            for type_arg in &self.type_args {
                let type_tag = self.parse_type_arg(type_arg)?;
                parsed.push(type_tag);
            }
            println!("   Type Args: {}", self.type_args.join(", "));
            parsed
        } else {
            vec![]
        };

        // Parse arguments
        let _args = if !self.args.is_empty() {
            let parsed = self.parse_args_vec(&self.args)?;
            println!("   Arguments: {} args provided", parsed.len());
            for (i, arg) in self.args.iter().enumerate() {
                println!("     [{}]: {}", i, arg);
            }
            parsed
        } else {
            vec![]
        };

        // Estimate gas
        let estimated_gas = 35_000 + (self.function.len() as u64 * 100);
        println!("\n⛽ Gas Estimation:");
        println!("   Estimated: {} units", estimated_gas);
        println!("   Limit: {} units", self.gas_limit);
        println!("   Total Cost: {} Mist", estimated_gas * self.gas_price);

        if self.dry_run {
            println!("\n🧪 Dry run mode - not executing");
            return Ok(());
        }

        // Create transaction
        println!("\n🔨 Creating transaction...");

        if let Some(ref wallet) = wallet {
            println!(
                "   🔑 Signing transaction with {} key...",
                wallet.curve_type
            );

            // In production, this would:
            // 1. Create ContractCall
            // 2. Sign with wallet private key
            // 3. Submit to RPC endpoint
            // 4. Wait for confirmation

            println!("   ⚠️  Not yet implemented: Blockchain submission");
        }

        println!("\n✅ Function call prepared!");
        println!("⚠️  Note: Blockchain submission not yet implemented");
        println!("   Use `execute_with_engine()` method for direct engine access");

        println!("\n💡 Next steps:");
        println!("   • Check transaction status");
        println!("   • View execution results on explorer");

        Ok(())
    }

    /// Parse a single type argument
    fn parse_type_arg(&self, type_arg: &str) -> Result<TypeTag> {
        let type_arg = type_arg.trim();

        // Parse type tag
        let type_tag = parser::parse_type_tag(type_arg)
            .with_context(|| format!("Failed to parse type argument: {}", type_arg))?;

        Ok(type_tag)
    }

    /// Parse function arguments from Vec<String>
    fn parse_args_vec(&self, args_vec: &[String]) -> Result<Vec<Vec<u8>>> {
        let mut result = Vec::new();

        for arg in args_vec {
            let arg = arg.trim();

            // Try to parse as different types
            let bytes = if arg.starts_with(r"0x") {
                // Hex address or bytes
                let hex_str = &arg[2..];

                // Check if it looks like an address (1-64 hex chars)
                if hex_str.len() <= 64 && hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
                    // Pad to 32 bytes for addresses
                    let padded = format!("{:0>64}", hex_str);
                    let addr = AccountAddress::from_hex_literal(&format!(r"0x{}", padded))
                        .with_context(|| format!("Failed to parse address: {}", arg))?;
                    bcs::to_bytes(&addr)?
                } else {
                    // Raw hex bytes
                    hex::decode(hex_str).with_context(|| format!("Failed to parse hex: {}", arg))?
                }
            } else if let Ok(num) = arg.parse::<u64>() {
                // u64 number
                bcs::to_bytes(&num)?
            } else if let Ok(num) = arg.parse::<u128>() {
                // u128 number
                bcs::to_bytes(&num)?
            } else if arg == "true" || arg == "false" {
                // Boolean
                let b = arg == "true";
                bcs::to_bytes(&b)?
            } else {
                // String
                bcs::to_bytes(arg)?
            };

            result.push(bytes);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_vec() {
        let call = Call {
            package: "0x1".to_string(),
            module: "coin".to_string(),
            function: "transfer".to_string(),
            sender: "0x1".to_string(),
            type_args: vec![],
            args: vec![],
            gas_limit: 200000,
            gas_price: 1000,
            password: None,
            skip_signature: true,
            rpc_endpoint: "http://localhost:9944".to_string(),
            dry_run: false,
        };

        // Test u64
        let args = call
            .parse_args_vec(&["1000".to_string(), "2000".to_string()])
            .unwrap();
        assert_eq!(args.len(), 2);

        // Test boolean
        let args = call
            .parse_args_vec(&["true".to_string(), "false".to_string()])
            .unwrap();
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn test_parse_type_arg() {
        let call = Call {
            package: "0x1".to_string(),
            module: "coin".to_string(),
            function: "transfer".to_string(),
            sender: "0x1".to_string(),
            type_args: vec![],
            args: vec![],
            gas_limit: 200000,
            gas_price: 1000,
            password: None,
            skip_signature: true,
            rpc_endpoint: "http://localhost:9944".to_string(),
            dry_run: false,
        };

        // Test parsing type argument
        let type_tag = call.parse_type_arg("u64").unwrap();
        assert!(matches!(type_tag, TypeTag::U64));
    }
}
