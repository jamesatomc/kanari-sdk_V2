module kanari_system::coin {
    use std::option;
    use std::ascii::String;
    use kanari_system::balance;
    use kanari_system::tx_context::TxContext;

    /// Minimal Coin resource wrapper (per token type)
    struct Coin<phantom T> has store, drop {
        value: u64,
    }

    /// Treasury: holds authority to mint into a Supply
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

    /// Create a new currency. This is a minimal implementation returning
    /// a `Treasury` and a `CurrencyMetadata` object. The `ctx` parameter is
    /// accepted to match expected caller signatures (e.g., genesis flows).
    public fun create_currency<T: drop>(
        _token: T,
        _decimals: u8,
        symbol: vector<u8>,
        name: vector<u8>,
        description: vector<u8>,
        _icon: option::Option<String>,
        _ctx: &mut TxContext,
    ): (Treasury<T>, CurrencyMetadata) {
        // Token witness is consumed automatically as it has drop ability
        (
            Treasury<T> {},
            CurrencyMetadata { symbol, name, description },
        )
    }

    /// Convert a treasury into a supply handle (consumes the treasury)
    public fun treasury_into_supply<T>(treasury: Treasury<T>): Supply<T> {
        let Treasury {} = treasury;
        Supply<T> { total: 0 }
    }

    /// Increase supply and return a Balance<T> representing newly minted tokens
    public fun increase_supply<T>(s: Supply<T>, amount: u64): balance::Balance<T> {
        let Supply { total } = s;
        let _new_total = total + amount;
        balance::create<T>(amount)
    }

    /// Destroy supply handle (no-op for minimal implementation)
    public fun destroy_supply<T>(s: Supply<T>) {
        let Supply { total: _ } = s;
    }

    /// Create a coin instance (value holder)
    public fun create_coin<T>(amount: u64): Coin<T> {
        Coin<T> { value: amount }
    }

    /// Get value
    public fun value<T>(c: &Coin<T>): u64 { c.value }

    /// Burn and return underlying value
    public fun burn<T>(c: Coin<T>): u64 {
        let Coin { value } = c;
        value
    }

}
