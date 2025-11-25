/// Kanari Coin - เหรียญหลักของระบบ Kanari
/// Module นี้เป็น main entry point สำหรับการจัดการเหรียญ Kanari
/// จะประกอบด้วยฟังก์ชันสำหรับลงทะเบียนเหรียญ Kanari ในขั้น genesis
module kanari_system::kanari {
    use kanari_system::balance::Balance;
    use kanari_system::coin;
    use kanari_system::transfer;
    use kanari_system::tx_context::{Self, TxContext};
    use std::option;

    const EAlreadyMinted: u64 = 0;
    /// Sender is not @0x0 the system address.
    const ENotSystemAddress: u64 = 1;

    #[allow(unused_const)]
    /// The amount of Mist per Kanari token based on the fact that mist is
    /// 10^-9 of a Kanari token
    const MIST_PER_KANARI: u64 = 1_000_000_000;

    #[allow(unused_const)]
    /// The total supply of Kanari denominated in whole Kanari tokens (10 Billion)
    const TOTAL_SUPPLY_KANARI: u64 = 10_000_000_000;

    /// The total supply of Kanari denominated in Mist (10 Billion * 10^9)
    const TOTAL_SUPPLY_MIST: u64 = 10_000_000_000_000_000_000;

    /// Name of the coin
    struct KANARI has drop {}

    #[allow(unused_function)]
    // Register the `KANARI` Coin to acquire its `Supply`.
    // This should be called only once during genesis creation.
    fun new(ctx: &mut TxContext): Balance<KANARI> {
        assert!(tx_context::sender(ctx) == @0x0, ENotSystemAddress);
        assert!(tx_context::epoch(ctx) == 0, EAlreadyMinted);

        let (treasury, metadata) = coin::create_currency(
            KANARI {},
            9,
            b"KANARI",
            b"KANARI",
            // TODO: add appropriate description and logo url
            b"",
            option::none(),
            ctx,
        );
        transfer::public_freeze_object(metadata);
        let supply = coin::treasury_into_supply(treasury);
        let total_kanari = coin::increase_supply(supply, TOTAL_SUPPLY_MIST);
        total_kanari
    }
}