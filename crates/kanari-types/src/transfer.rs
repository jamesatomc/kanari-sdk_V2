use anyhow::{Result, Context};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
};
use serde::{Serialize, Deserialize};

/// Transfer record structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferRecord {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl TransferRecord {
    /// Create a new transfer record
    pub fn new(from: String, to: String, amount: u64) -> Self {
        Self {
            from,
            to,
            amount,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create transfer record from AccountAddress
    pub fn from_addresses(from: AccountAddress, to: AccountAddress, amount: u64) -> Self {
        Self::new(format!("{}", from), format!("{}", to), amount)
    }
}

/// Transfer validation utilities
pub struct TransferValidator;

impl TransferValidator {
    /// Validate transfer amount and addresses
    pub fn validate(from: u64, to: u64, amount: u64) -> Result<bool> {
        // Basic validation rules:
        // 1. Amount must be greater than 0
        // 2. From and to addresses must be different
        Ok(amount > 0 && from != to)
    }

    /// Validate transfer with addresses
    pub fn validate_addresses(from: &AccountAddress, to: &AccountAddress, amount: u64) -> Result<bool> {
        Ok(amount > 0 && from != to)
    }
}

/// Transfer module constants and utilities
pub struct TransferModule;

impl TransferModule {
    pub const SYSTEM_ADDRESS: &'static str = "0x1";
    pub const TRANSFER_MODULE: &'static str = "transfer";

    /// Get the module ID for system::transfer
    pub fn get_module_id() -> Result<ModuleId> {
        let address = AccountAddress::from_hex_literal(Self::SYSTEM_ADDRESS)
            .context("Invalid system address")?;
        
        let module_name = Identifier::new(Self::TRANSFER_MODULE)
            .context("Invalid transfer module name")?;
        
        Ok(ModuleId::new(address, module_name))
    }

    /// Get function names used in transfer module
    pub fn function_names() -> TransferFunctions {
        TransferFunctions {
            is_valid_amount: "is_valid_amount",
            create_transfer: "create_transfer",
            get_amount: "get_amount",
            get_from: "get_from",
            get_to: "get_to",
        }
    }
}

/// Transfer module function names
pub struct TransferFunctions {
    pub is_valid_amount: &'static str,
    pub create_transfer: &'static str,
    pub get_amount: &'static str,
    pub get_from: &'static str,
    pub get_to: &'static str,
}

/// Convert AccountAddress to u64 for simplified addressing
pub fn address_to_u64(address: &AccountAddress) -> u64 {
    let bytes = address.to_vec();
    let mut result = 0u64;
    for (i, &byte) in bytes.iter().take(8).enumerate() {
        result |= (byte as u64) << (i * 8);
    }
    result
}

/// Convert u64 to AccountAddress
pub fn u64_to_address(value: u64) -> AccountAddress {
    let mut bytes = [0u8; 32];
    for i in 0..8 {
        bytes[i] = ((value >> (i * 8)) & 0xFF) as u8;
    }
    AccountAddress::new(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_record_creation() {
        let record = TransferRecord::new(
            "0x1".to_string(),
            "0x2".to_string(),
            1000,
        );
        assert_eq!(record.from, "0x1");
        assert_eq!(record.to, "0x2");
        assert_eq!(record.amount, 1000);
        assert!(record.timestamp > 0);
    }

    #[test]
    fn test_transfer_validator() {
        // Valid transfer
        assert!(TransferValidator::validate(100, 200, 500).unwrap());
        
        // Invalid: zero amount
        assert!(!TransferValidator::validate(100, 200, 0).unwrap());
        
        // Invalid: same address
        assert!(!TransferValidator::validate(100, 100, 500).unwrap());
    }

    #[test]
    fn test_get_transfer_module_id() {
        let module_id = TransferModule::get_module_id();
        assert!(module_id.is_ok());
        
        let module_id = module_id.unwrap();
        assert_eq!(module_id.name().as_str(), "transfer");
    }

    #[test]
    fn test_address_conversion() {
        let value = 12345u64;
        let address = u64_to_address(value);
        let converted_back = address_to_u64(&address);
        assert_eq!(value, converted_back);
    }
}
