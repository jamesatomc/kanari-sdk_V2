//! Keystore management functionality
//!
//! This module handles the kanari.keystore format for secure storage of wallet information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

use kanari_common::get_kanari_config_path;

use crate::encryption::EncryptedData;

/// Errors related to keystore operations
#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid keystore format")]
    InvalidFormat,

    #[error("Password verification failed")]
    PasswordVerificationFailed,

    #[error("Keystore is locked")]
    Locked,

    #[error("Keystore is corrupted: {0}")]
    Corrupted(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Backup error: {0}")]
    BackupError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Structure representing the keystore file
#[derive(Serialize, Deserialize, Default)]
pub struct Keystore {
    /// Individual wallet keys by address
    pub keys: HashMap<String, EncryptedData>,

    /// Mnemonic phrase information
    pub mnemonic: MnemonicStore,

    /// Temporary session keys
    pub session_keys: HashMap<String, String>,

    /// Hashed master password for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,

    /// Whether the password is empty
    #[serde(default)]
    pub is_password_empty: bool,

    /// Version of the keystore format
    #[serde(default = "default_keystore_version")]
    pub version: String,

    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<u64>,
}

fn default_keystore_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Structure for storing mnemonic phrases
#[derive(Serialize, Deserialize, Default)]
pub struct MnemonicStore {
    /// List of addresses derived from the mnemonic
    pub addresses: Vec<String>,

    /// Encrypted mnemonic phrase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic_phrase_encryption: Option<EncryptedData>,
}

impl Keystore {
    /// Load keystore from disk
    pub fn load() -> Result<Self, KeystoreError> {
        let keystore_path = get_keystore_path();

        if !keystore_path.exists() {
            return Ok(Keystore::default());
        }

        // Load the keystore data
        let keystore_data = fs::read_to_string(keystore_path)?;
        let mut keystore: Keystore = serde_json::from_str(&keystore_data)?;

        // Upgrade any keys that might be using the old format
        for (_, encrypted_data) in keystore.keys.iter_mut() {
            *encrypted_data = crate::encryption::upgrade_encrypted_data(encrypted_data.clone());
        }

        // Save if any changes were made (conversion from array to base64)
        keystore.save()?;

        Ok(keystore)
    }

    /// Save keystore to disk
    pub fn save(&mut self) -> Result<(), KeystoreError> {
        let keystore_path = get_keystore_path();
        let keystore_dir = keystore_path
            .parent()
            .ok_or_else(|| KeystoreError::InvalidPath("Invalid keystore path".to_string()))?;

        // Create directory if it doesn't exist
        if !keystore_dir.exists() {
            fs::create_dir_all(keystore_dir)?;
        }

        // Update last modified timestamp
        self.last_modified = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| KeystoreError::InvalidPath(format!("System time error: {}", e)))?
                .as_secs(),
        );

        let keystore_data = serde_json::to_string_pretty(self)?;
        fs::write(keystore_path, keystore_data)?;

        Ok(())
    }

    /// Add a wallet to the keystore
    pub fn add_wallet(
        &mut self,
        address: &str,
        encrypted_data: EncryptedData,
    ) -> Result<(), KeystoreError> {
        self.keys.insert(address.to_string(), encrypted_data);
        self.save()?;
        Ok(())
    }

    /// Get a wallet from the keystore
    pub fn get_wallet(&self, address: &str) -> Option<&EncryptedData> {
        self.keys.get(address)
    }

    /// Remove a wallet from the keystore
    pub fn remove_wallet(&mut self, address: &str) -> Result<(), KeystoreError> {
        if self.keys.remove(address).is_none() {
            return Err(KeystoreError::KeyNotFound(address.to_string()));
        }

        // Also remove from mnemonic addresses if present
        self.mnemonic.addresses.retain(|addr| addr != address);

        self.save()?;
        Ok(())
    }

    /// Check if a wallet exists in the keystore
    pub fn wallet_exists(&self, address: &str) -> bool {
        self.keys.contains_key(address)
    }

    /// List all wallets in the keystore
    pub fn list_wallets(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    /// Set encrypted mnemonic phrase
    pub fn set_mnemonic(
        &mut self,
        encrypted_mnemonic: EncryptedData,
        addresses: Vec<String>,
    ) -> Result<(), KeystoreError> {
        self.mnemonic.mnemonic_phrase_encryption = Some(encrypted_mnemonic);
        self.mnemonic.addresses = addresses;
        self.save()?;
        Ok(())
    }

    /// Get encrypted mnemonic phrase
    pub fn get_mnemonic(&self) -> Option<&EncryptedData> {
        self.mnemonic.mnemonic_phrase_encryption.as_ref()
    }

    /// Get addresses derived from mnemonic
    pub fn get_mnemonic_addresses(&self) -> &Vec<String> {
        &self.mnemonic.addresses
    }

    /// Add address to mnemonic-derived addresses
    pub fn add_mnemonic_address(&mut self, address: &str) -> Result<(), KeystoreError> {
        if !self.mnemonic.addresses.contains(&address.to_string()) {
            self.mnemonic.addresses.push(address.to_string());
            self.save()?;
        }
        Ok(())
    }

    /// Remove mnemonic and all associated data
    pub fn remove_mnemonic(&mut self) -> Result<(), KeystoreError> {
        self.mnemonic.mnemonic_phrase_encryption = None;
        self.mnemonic.addresses.clear();
        self.save()?;
        Ok(())
    }

    /// Add session key
    pub fn add_session_key(&mut self, key: &str, value: &str) -> Result<(), KeystoreError> {
        self.session_keys.insert(key.to_string(), value.to_string());
        self.save()?;
        Ok(())
    }

    /// Get session key
    pub fn get_session_key(&self, key: &str) -> Option<&String> {
        self.session_keys.get(key)
    }

    /// Remove session key
    pub fn remove_session_key(&mut self, key: &str) -> Result<(), KeystoreError> {
        self.session_keys.remove(key);
        self.save()?;
        Ok(())
    }

    /// Clear all session keys
    pub fn clear_session_keys(&mut self) -> Result<(), KeystoreError> {
        self.session_keys.clear();
        self.save()?;
        Ok(())
    }

    /// Check if mnemonic exists
    pub fn has_mnemonic(&self) -> bool {
        self.mnemonic.mnemonic_phrase_encryption.is_some()
    }

    /// Validate keystore integrity
    pub fn validate(&self) -> Result<(), KeystoreError> {
        // Check version compatibility
        if self.version.is_empty() {
            return Err(KeystoreError::InvalidFormat);
        }

        // Validate all encrypted data entries
        for (address, encrypted_data) in &self.keys {
            match encrypted_data.get_ciphertext() {
                Ok(ciphertext) if ciphertext.is_empty() => {
                    return Err(KeystoreError::Corrupted(format!(
                        "Empty ciphertext for address: {}",
                        address
                    )));
                }
                Err(e) => {
                    return Err(KeystoreError::Corrupted(format!(
                        "Invalid ciphertext for address {}: {}",
                        address, e
                    )));
                }
                _ => {}
            }

            match encrypted_data.get_nonce() {
                Ok(nonce) if nonce.is_empty() => {
                    return Err(KeystoreError::Corrupted(format!(
                        "Empty nonce for address: {}",
                        address
                    )));
                }
                Err(e) => {
                    return Err(KeystoreError::Corrupted(format!(
                        "Invalid nonce for address {}: {}",
                        address, e
                    )));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Get keystore statistics
    pub fn statistics(&self) -> KeystoreStatistics {
        KeystoreStatistics {
            total_keys: self.keys.len(),
            has_mnemonic: self.has_mnemonic(),
            mnemonic_addresses: self.mnemonic.addresses.len(),
            session_keys: self.session_keys.len(),
            version: self.version.clone(),
            last_modified: self.last_modified,
        }
    }
}

/// Keystore statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreStatistics {
    pub total_keys: usize,
    pub has_mnemonic: bool,
    pub mnemonic_addresses: usize,
    pub session_keys: usize,
    pub version: String,
    pub last_modified: Option<u64>,
}

/// Get path to the keystore file
pub fn get_keystore_path() -> PathBuf {
    let mut keystore_dir = get_kanari_config_path();
    // Remove 'kanari.yaml' from the path and add 'kanari.keystore'
    keystore_dir.pop();
    keystore_dir.push("kanari.keystore");
    keystore_dir
}

/// Check if keystore file exists
pub fn keystore_exists() -> bool {
    get_keystore_path().exists()
}
