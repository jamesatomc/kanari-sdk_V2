// Genesis helper to initialize the system and mint KANARI to dev address
// This is a small, test-only initialization routine used by local runs
// to populate the developer account with the initial supply.
#[test_only]
module kanari_system::genesis {
	use std::option;
	use kanari_system::tx_context;
	use kanari_system::coin;
	use kanari_system::kanari::{Self, KANARI};

	/// Mint the full kanari supply to the developer address `@0x1`.
	/// Note: uses `tx_context::dummy()` (test-only) as a genesis context.
	public fun init_for_dev() {
		// Create a dummy tx context (genesis context)
		let ctx = tx_context::dummy();

		// Construct the token witness and create the currency
		let witness = KANARI {};

		let (cap, _metadata) = coin::create_currency(
			witness,
			9u8,
			b"KANARI",
			b"KANARI",
			b"Genesis Kanari Token",
			option::none(),
			&mut ctx,
		);
		let treasury_cap = cap;
		// Total supply in MIST: 10_000_000_000 * 10^9 = 10_000_000_000_000_000_000
		let total_supply_mist: u64 = 10_000_000_000_000_000_000u64;

		// Mint entire supply and transfer to developer
		coin::mint_and_transfer(&mut treasury_cap, total_supply_mist, @0x3603a1c5737316534fbb1cc0fa599258e401823059f3077d2a8d86a998825739, &mut ctx);
	}
}

