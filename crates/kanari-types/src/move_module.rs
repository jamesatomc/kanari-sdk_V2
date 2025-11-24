use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::ModuleId,
};

/// System module addresses and names
pub struct SystemModules;

impl SystemModules {
    /// System address (0x2)
    pub const SYSTEM_ADDRESS: &'static str = "0x2";
    
    /// Module names
    pub const TRANSFER_MODULE: &'static str = "transfer";
    pub const ACCOUNT_MODULE: &'static str = "account";
    
    /// Get the module ID for system::transfer
    pub fn get_transfer_module_id() -> Result<ModuleId> {
        Self::get_module_id(Self::SYSTEM_ADDRESS, Self::TRANSFER_MODULE)
    }

    /// Get the module ID for system::account
    pub fn get_account_module_id() -> Result<ModuleId> {
        Self::get_module_id(Self::SYSTEM_ADDRESS, Self::ACCOUNT_MODULE)
    }

    /// Get a custom module ID
    pub fn get_module_id(address: &str, module_name: &str) -> Result<ModuleId> {
        let addr = AccountAddress::from_hex_literal(address)?;
        let name = Identifier::new(module_name)?;
        Ok(ModuleId::new(addr, name))
    }

    /// Get system address (0x2)
    pub fn system_address() -> Result<AccountAddress> {
        Ok(AccountAddress::from_hex_literal(Self::SYSTEM_ADDRESS)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_transfer_module_id() {
        let module_id = SystemModules::get_transfer_module_id();
        assert!(module_id.is_ok());
        let module_id = module_id.unwrap();
        assert_eq!(module_id.name().as_str(), "transfer");
    }

    #[test]
    fn test_get_account_module_id() {
        let module_id = SystemModules::get_account_module_id();
        assert!(module_id.is_ok());
    }

    #[test]
    fn test_system_address() {
        let addr = SystemModules::system_address();
        assert!(addr.is_ok());
    }
}
