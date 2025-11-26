use anyhow::{Context, Result};
use move_core_types::account_address::AccountAddress;

use crate::balance::BalanceRecord;
use crate::kanari::KanariModule;

/// Developer address literal used for genesis allocation
pub const DEV_ADDRESS_LITERAL: &str =
    "0x3603a1c5737316534fbb1cc0fa599258e401823059f3077d2a8d86a998825739";

/// Return the developer address as a Move `AccountAddress`.
pub fn dev_account_address() -> Result<AccountAddress> {
    AccountAddress::from_hex_literal(DEV_ADDRESS_LITERAL).context("Invalid dev address literal")
}

/// Return the initial balance (in MIST) allocated to the developer address at genesis.
pub fn dev_initial_balance() -> BalanceRecord {
    BalanceRecord::new(KanariModule::TOTAL_SUPPLY_MIST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_address_parse() {
        let addr = dev_account_address().expect("parse dev addr");
        let expected =
            AccountAddress::from_hex_literal(DEV_ADDRESS_LITERAL).expect("parse dev addr expected");
        assert_eq!(addr, expected);
    }

    #[test]
    fn test_dev_initial_balance() {
        let b = dev_initial_balance();
        assert_eq!(b.value, KanariModule::TOTAL_SUPPLY_MIST);
    }
}
