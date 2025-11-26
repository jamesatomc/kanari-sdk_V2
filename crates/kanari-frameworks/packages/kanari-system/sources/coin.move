module kanari_system::coin {
    use std::option;
    use std::ascii::String;
    use kanari_system::balance::{Self, Balance};
    use kanari_system::tx_context::TxContext;
    use kanari_system::transfer;

    /// Coin resource wrapper with balance
    struct Coin<phantom T> has store, drop {
        balance: Balance<T>,
    }

    /// Capability allowing the bearer to mint and burn coins
    struct TreasuryCap<phantom T> has store, drop {
        total_supply: u64,
    }

    /// Treasury: holds authority to mint into a Supply (deprecated, use TreasuryCap)
    struct Treasury<phantom T> has store, drop {
    }

    /// Supply: mutable minting handle consumed to create balances
    struct Supply<phantom T> has store {
        total: u64,
    }

    /// Simple metadata for a currency
    struct CurrencyMetadata has store, drop {
        symbol: vector<u8>,
        name: vector<u8>,
        description: vector<u8>,
    }

    /// Create a new currency with TreasuryCap for minting control
    public fun create_currency<T: drop>(
        witness: T,
        decimals: u8,
        symbol: vector<u8>,
        name: vector<u8>,
        description: vector<u8>,
        icon_url: option::Option<String>,
        ctx: &mut TxContext,
    ): (TreasuryCap<T>, CurrencyMetadata) {
        // Token witness is consumed automatically as it has drop ability
        let _ = witness;
        let _ = decimals;
        let _ = icon_url;
        let _ = ctx;
        
        let treasury_cap = TreasuryCap<T> {
            total_supply: 0,
        };
        
        (
            treasury_cap,
            CurrencyMetadata { symbol, name, description },
        )
    }

    /// Mint new coins using TreasuryCap
    public fun mint<T>(
        cap: &mut TreasuryCap<T>,
        amount: u64,
        _ctx: &mut TxContext,
    ): Coin<T> {
        cap.total_supply = cap.total_supply + amount;
        Coin {
            balance: balance::create(amount),
        }
    }

    /// Mint and transfer to recipient
    public fun mint_and_transfer<T>(
        cap: &mut TreasuryCap<T>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext,
    ) {
        let coin = mint(cap, amount, ctx);
        transfer::public_transfer(coin, recipient);
    }

    /// Burn coins, decreasing total supply
    public fun burn<T>(cap: &mut TreasuryCap<T>, coin: Coin<T>): u64 {
        let Coin { balance } = coin;
        let value = balance::destroy(balance);
        cap.total_supply = cap.total_supply - value;
        value
    }

    /// Get total supply from TreasuryCap
    public fun total_supply<T>(cap: &TreasuryCap<T>): u64 {
        cap.total_supply
    }

    /// Get coin value
    public fun value<T>(coin: &Coin<T>): u64 {
        balance::value(&coin.balance)
    }

    /// Split a coin into two
    public fun split<T>(coin: &mut Coin<T>, amount: u64, ctx: &mut TxContext): Coin<T> {
        let _ = ctx;
        Coin {
            balance: balance::split(&mut coin.balance, amount),
        }
    }

    /// Join two coins together
    public fun join<T>(coin: &mut Coin<T>, other: Coin<T>) {
        let Coin { balance } = other;
        balance::merge(&mut coin.balance, balance);
    }

    /// Convert a treasury into a supply handle (deprecated)
    public fun treasury_into_supply<T>(treasury: Treasury<T>): Supply<T> {
        let Treasury {} = treasury;
        Supply<T> { total: 0 }
    }

    /// Increase supply (deprecated)
    public fun increase_supply<T>(s: Supply<T>, amount: u64): Balance<T> {
        let Supply { total } = s;
        let _new_total = total + amount;
        balance::create(amount)
    }

    /// Destroy supply handle (deprecated)
    public fun destroy_supply<T>(s: Supply<T>) {
        let Supply { total: _ } = s;
    }

}
