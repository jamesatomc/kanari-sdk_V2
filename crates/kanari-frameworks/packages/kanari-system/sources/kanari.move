/// Coin<KARI> is the token used to pay for gas in Kari.
/// It has 9 decimals, and the smallest unit (10^-9) is called "KA".
module kanari_system::kanari {
    use std::option;
    use kanari_system::tx_context::TxContext;
    use kanari_system::transfer;
    use kanari_system::coin;
    use kanari_system::coin::{Coin, TreasuryCap};


    /// The amount of Mist per Kanari token based on the fact that mist is
    /// 10^-9 of a Kanari token
    const MIST_PER_KANARI: u64 = 1_000_000_000;

    /// The total supply of Kanari denominated in whole Kanari tokens (10 Billion)
    const TOTAL_SUPPLY_KANARI: u64 = 10_000_000_000;

    /// The total supply of Kanari denominated in Mist (10 Billion * 10^9)
    const TOTAL_SUPPLY_MIST: u64 = 10_000_000_000_000_000_000;

    /// Name of the coin
    struct KANARI has drop {}

    /// Initialize the KANARI currency with one-time witness
    /// Mint the full `TOTAL_SUPPLY_MIST` to the genesis address `@0x1` and
    /// drop the treasury capability so no further minting is possible.
    public fun init(witness: KANARI, ctx: &mut TxContext) {
        let (treasury_cap, metadata) = coin::create_currency(
            witness,
            9,
            b"KANARI",
            b"KANARI",
            // TODO: add appropriate description and logo url
            b"",
            option::none(),
            ctx
        );
        // Mint the entire supply in Mist and send to genesis address
        // `treasury_cap` is returned by `create_currency` and can be used
        // directly for mutable borrows by the callee; re-binding without
        // `mut` avoids parser issues with older Move grammars.
        // Rebind to a short name for clarity when passing by mutable reference
        let cap = treasury_cap;
        let all_coins = coin::mint(&mut cap, TOTAL_SUPPLY_MIST, ctx);
        transfer::public_transfer(all_coins, @0x3603a1c5737316534fbb1cc0fa599258e401823059f3077d2a8d86a998825739);

        // Freeze metadata (no-op placeholder in this package)
        transfer::public_freeze_object(metadata);

        // `treasury_cap` goes out of scope here and will be dropped.
        // Because the capability has `drop`, dropping it prevents further minting.
    }

    /// KARI tokens to the treasury
    public fun transfer(c: coin::Coin<KANARI>, recipient: address) {
        transfer::public_transfer(c, recipient)
    }

    /// Burns KANARI tokens, decreasing total supply
    public fun burn(treasury_cap: &mut TreasuryCap<KANARI>, coin: Coin<KANARI>) {
        coin::burn(treasury_cap, coin);
    }
}