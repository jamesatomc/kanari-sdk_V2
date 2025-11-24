mod compiler;
mod packages_config;

use anyhow::Result;
use clap::Parser;
use std::env;
use packages_config::get_package_configs;

#[derive(Parser)]
#[command(name = "packages")]
#[command(about = "Kanari Package Compiler", long_about = None)]
struct Cli {
    /// Package version to compile (default: 1)
    #[arg(long, default_value = "1")]
    version: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("🚀 Kanari Package Compiler");
    println!("==========================\n");
    println!("📌 Version: {}\n", cli.version);

    // Get the workspace root (go up from packages directory)
    let current_dir = env::current_dir()?;
    
    // Find the actual packages directory
    let packages_dir = if current_dir.ends_with("packages") {
        current_dir.clone()
    } else {
        current_dir.join("crates/packages")
    };
    
    // Output directory for compiled packages (released/)
    let output_dir = packages_dir.join("released");
    
    println!("📁 Packages directory: {:?}", packages_dir);
    println!("📁 Output directory: {:?}\n", output_dir);

    // Get all package configurations
    let package_configs = get_package_configs();
    let mut compiled_count = 0;
    let mut failed_count = 0;

    // Compile all configured packages
    for config in package_configs {
        let package_dir = packages_dir.join(config.directory);
        
        if !package_dir.exists() {
            eprintln!("⚠️  Package directory not found: {:?}\n", package_dir);
            continue;
        }

        // Convert AccountAddress to short hex format (0x1, 0x2, 0x3, etc.)
        // AccountAddress is 32 bytes stored as big-endian
        // For simple addresses like 0x1, 0x2, the value is in the last byte
        let addr_bytes = config.address.to_vec();
        let mut value = 0u64;
        // Read up to last 8 bytes in reverse (big-endian to u64)
        let start = addr_bytes.len().saturating_sub(8);
        for &byte in &addr_bytes[start..] {
            value = (value << 8) | (byte as u64);
        }
        let address_str = format!("0x{:x}", value);
        
        println!("Compiling {} ({})...", config.name, address_str);
        
        match compiler::compile_package(&package_dir, &output_dir, &cli.version, &address_str) {
            Ok(output_file) => {
                println!("✅ Successfully compiled {}", config.name);
                println!("   Output: {:?}\n", output_file);
                compiled_count += 1;
            }
            Err(e) => {
                eprintln!("❌ Failed to compile {}: {}", config.name, e);
                eprintln!("   Error details: {:?}\n", e);
                failed_count += 1;
            }
        }
    }

    // Summary
    println!("\n✨ Compilation Summary:");
    println!("   ✅ Successful: {}", compiled_count);
    if failed_count > 0 {
        println!("   ❌ Failed: {}", failed_count);
    }
    
    Ok(())
}
