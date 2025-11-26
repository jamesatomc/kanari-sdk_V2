/// Coin<KARI> is the token used to pay for gas in Kari.
/// It has 9 decimals, and the smallest unit (10^-9) is called "KA".
module kanari_system::kanari {
    use std::option;
    use kanari_system::tx_context::{Self, TxContext};
    use kanari_system::transfer;
    use kanari_system::coin::{Self, Coin, TreasuryCap};


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
    fun init(witness: KANARI, ctx: &mut TxContext) {
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
        transfer::public_freeze_object(metadata);
        // Store treasury_cap for later minting
        // In production, this would be transferred to governance or stored properly
        transfer::public_transfer(treasury_cap, tx_context::sender(ctx));
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