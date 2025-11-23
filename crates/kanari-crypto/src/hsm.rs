//! Hardware Security Module (HSM) support infrastructure
//!
//! This module provides interfaces and abstractions for integrating with
//! Hardware Security Modules for enhanced key security.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors related to HSM operations
#[derive(Error, Debug)]
pub enum HsmError {
    #[error("HSM not available: {0}")]
    NotAvailable(String),

    #[error("HSM operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid HSM configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Key not found in HSM: {0}")]
    KeyNotFound(String),

    #[error("HSM authentication failed")]
    AuthenticationFailed,

    #[error("Unsupported HSM operation: {0}")]
    UnsupportedOperation(String),
}

/// HSM provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HsmProvider {
    /// Software-based HSM (for development/testing)
    Software,
    /// YubiKey HSM
    YubiKey,
    /// AWS CloudHSM
    AwsCloudHsm,
    /// Azure Key Vault
    AzureKeyVault,
    /// Generic PKCS#11 interface
    Pkcs11,
}

/// HSM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// Provider type
    pub provider: HsmProvider,
    /// Connection string or device path
    pub connection: String,
    /// Optional authentication token/PIN
    #[serde(skip_serializing)]
    pub auth_token: Option<String>,
    /// Enable HSM for all operations
    pub enabled: bool,
}

impl Default for HsmConfig {
    fn default() -> Self {
        Self {
            provider: HsmProvider::Software,
            connection: String::new(),
            auth_token: None,
            enabled: false,
        }
    }
}

/// Trait for HSM operations
pub trait HsmInterface {
    /// Initialize connection to HSM
    fn connect(&mut self, config: &HsmConfig) -> Result<(), HsmError>;

    /// Disconnect from HSM
    fn disconnect(&mut self) -> Result<(), HsmError>;

    /// Check if HSM is available and connected
    fn is_connected(&self) -> bool;

    /// Generate a new key in HSM
    fn generate_key(&mut self, key_id: &str, algorithm: &str) -> Result<Vec<u8>, HsmError>;

    /// Sign data using HSM-stored key
    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, HsmError>;

    /// Verify signature using HSM-stored key
    fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool, HsmError>;

    /// Delete key from HSM
    fn delete_key(&mut self, key_id: &str) -> Result<(), HsmError>;

    /// List all keys in HSM
    fn list_keys(&self) -> Result<Vec<String>, HsmError>;

    /// Export public key (private key never leaves HSM)
    fn export_public_key(&self, key_id: &str) -> Result<Vec<u8>, HsmError>;
}

/// Software-based HSM implementation (for development/testing)
#[derive(Debug)]
pub struct SoftwareHsm {
    connected: bool,
    keys: std::collections::HashMap<String, Vec<u8>>,
}

impl Default for SoftwareHsm {
    fn default() -> Self {
        Self {
            connected: false,
            keys: std::collections::HashMap::new(),
        }
    }
}

impl HsmInterface for SoftwareHsm {
    fn connect(&mut self, config: &HsmConfig) -> Result<(), HsmError> {
        if config.provider != HsmProvider::Software {
            return Err(HsmError::InvalidConfiguration(
                "Expected Software HSM provider".to_string(),
            ));
        }
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), HsmError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn generate_key(&mut self, key_id: &str, _algorithm: &str) -> Result<Vec<u8>, HsmError> {
        if !self.connected {
            return Err(HsmError::NotAvailable("HSM not connected".to_string()));
        }

        // Generate a random key (32 bytes for demonstration)
        use rand::RngCore;
        let mut key = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);

        self.keys.insert(key_id.to_string(), key.clone());
        Ok(key)
    }

    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, HsmError> {
        if !self.connected {
            return Err(HsmError::NotAvailable("HSM not connected".to_string()));
        }

        let _key = self
            .keys
            .get(key_id)
            .ok_or_else(|| HsmError::KeyNotFound(key_id.to_string()))?;

        // Placeholder: In real implementation, use the key to sign
        Ok(data.to_vec())
    }

    fn verify(&self, key_id: &str, _data: &[u8], _signature: &[u8]) -> Result<bool, HsmError> {
        if !self.connected {
            return Err(HsmError::NotAvailable("HSM not connected".to_string()));
        }

        if !self.keys.contains_key(key_id) {
            return Err(HsmError::KeyNotFound(key_id.to_string()));
        }

        // Placeholder: In real implementation, verify signature
        Ok(true)
    }

    fn delete_key(&mut self, key_id: &str) -> Result<(), HsmError> {
        if !self.connected {
            return Err(HsmError::NotAvailable("HSM not connected".to_string()));
        }

        self.keys
            .remove(key_id)
            .ok_or_else(|| HsmError::KeyNotFound(key_id.to_string()))?;

        Ok(())
    }

    fn list_keys(&self) -> Result<Vec<String>, HsmError> {
        if !self.connected {
            return Err(HsmError::NotAvailable("HSM not connected".to_string()));
        }

        Ok(self.keys.keys().cloned().collect())
    }

    fn export_public_key(&self, key_id: &str) -> Result<Vec<u8>, HsmError> {
        if !self.connected {
            return Err(HsmError::NotAvailable("HSM not connected".to_string()));
        }

        let key = self
            .keys
            .get(key_id)
            .ok_or_else(|| HsmError::KeyNotFound(key_id.to_string()))?;

        // Placeholder: In real implementation, derive public key
        Ok(key.clone())
    }
}

/// Factory function to create HSM interface based on provider
pub fn create_hsm(provider: HsmProvider) -> Result<Box<dyn HsmInterface>, HsmError> {
    match provider {
        HsmProvider::Software => Ok(Box::new(SoftwareHsm::default())),
        _ => Err(HsmError::UnsupportedOperation(format!(
            "HSM provider {:?} not yet implemented. Currently only Software HSM is supported.",
            provider
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_hsm_lifecycle() {
        let mut hsm = SoftwareHsm::default();
        assert!(!hsm.is_connected());

        let config = HsmConfig {
            provider: HsmProvider::Software,
            connection: "memory".to_string(),
            auth_token: None,
            enabled: true,
        };

        hsm.connect(&config).unwrap();
        assert!(hsm.is_connected());

        let public_key = hsm.generate_key("test-key", "Ed25519").unwrap();
        assert!(!public_key.is_empty());

        let keys = hsm.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "test-key");

        hsm.delete_key("test-key").unwrap();
        let keys = hsm.list_keys().unwrap();
        assert_eq!(keys.len(), 0);

        hsm.disconnect().unwrap();
        assert!(!hsm.is_connected());
    }
}
