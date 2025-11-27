use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::str::FromStr;
use kanari_types::address::Address;
use kanari_crypto::{
    keys::{generate_keypair, generate_mnemonic, keypair_from_mnemonic, CurveType},
    wallet::{list_wallet_files, load_wallet, save_wallet, Wallet}, // added Wallet
};

/// Kanari - A Move-based money transfer system
#[derive(Parser)]
#[command(name = "kanari")]
#[command(about = "Money transfer system using Move language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet with kanari-crypto
    CreateWallet {
        /// Password for wallet encryption
        #[arg(short, long)]
        password: String,
        /// Curve type (ed25519, k256, p256, dilithium2, dilithium3, dilithium5)
        #[arg(short, long, default_value = "ed25519")]
        curve: String,
        /// Number of seed words (12 or 24)
        #[arg(short, long, default_value = "12")]
        words: usize,
    },
    /// Load an existing wallet
    LoadWallet {
        /// Wallet address to load
        #[arg(short, long)]
        address: String,
        /// Password to decrypt wallet
        #[arg(short, long)]
        password: String,
    },
    /// List all wallets with balances
    ListWallets,
    /// Show detailed wallet information
    WalletInfo {
        /// Wallet address
        #[arg(short, long)]
        address: String,
        /// Password to decrypt wallet
        #[arg(short, long)]
        password: String,
        /// Show private key and seed phrase (dangerous!)
        #[arg(long, default_value = "false")]
        show_secrets: bool,
    },
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	match cli.command {
        Commands::CreateWallet { password, curve, words } => {
            let curve_type = match curve.to_lowercase().as_str() {
                "ed25519" => CurveType::Ed25519,
                "k256" | "secp256k1" => CurveType::K256,
                "p256" | "secp256r1" => CurveType::P256,
                "dilithium2" => CurveType::Dilithium2,
                "dilithium3" => CurveType::Dilithium3,
                "dilithium5" => CurveType::Dilithium5,
                "sphincs+" | "sphincsplus" => CurveType::SphincsPlusSha256Robust,
                "ed25519+dilithium3" | "ed25519_dilithium3" => CurveType::Ed25519Dilithium3,
                "k256+dilithium3" | "k256_dilithium3" => CurveType::K256Dilithium3,
                other => {
                    println!("Unknown curve '{}', falling back to Ed25519", other);
                    CurveType::Ed25519
                }
            };

            // For classical curves we can derive from a mnemonic; for PQC/hybrid generate directly
            let (private_key, address_str, seed_phrase) = if curve_type.is_post_quantum() || curve_type.is_hybrid() {
                let kp = generate_keypair(curve_type)
                    .context("Failed to generate keypair")?;
                (kp.private_key, kp.address, String::new())
            } else {
                let mnemonic = generate_mnemonic(words)
                    .context("Failed to generate mnemonic")?;
                let kp = keypair_from_mnemonic(&mnemonic, curve_type, "")
                    .context("Failed to derive keypair from mnemonic")?;
                (kp.private_key, kp.address, mnemonic)
            };

            let address = Address::from_str(&address_str)
                .context("Generated invalid address")?;

            // Save wallet
            save_wallet(&address, &private_key, &seed_phrase, &password, curve_type)
                .context("Failed to save wallet")?;

            println!("Created wallet: {}", address_str);
            if !seed_phrase.is_empty() {
                println!("Seed phrase: {}", seed_phrase);
            }

            Ok(())
        }

		Commands::LoadWallet { address, password } => {
            let wallet: Wallet = load_wallet(&address, &password)
                .context("Failed to load wallet")?;
            println!("Wallet loaded: {} (curve: {})", address, wallet.curve_type);
			Ok(())
		}

		Commands::ListWallets => {
			let wallets = list_wallet_files()
				.context("Failed to list wallets")?;
			println!("Found {} wallets", wallets.len());
			Ok(())
		}

		Commands::WalletInfo { address, password, show_secrets } => {
            let wallet = load_wallet(&address, &password)
                .context("Failed to load wallet")?;
            println!("Wallet info for {}", address);
            if show_secrets {
                println!("Private key: {}", wallet.private_key);
                println!("Seed phrase: {}", wallet.seed_phrase);
            } else {
                println!("Address: {}", wallet.address.to_string());
            }
            Ok(())
		}

	}
}
