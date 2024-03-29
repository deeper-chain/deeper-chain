
//! Autogenerated weights for `pallet_credit`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `yubo-X400`, CPU: `AMD Ryzen 7 PRO 4750G with Radeon Graphics`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/debug/deeper-chain
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_credit
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/credit/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_credit`.
pub trait WeightInfo {
	fn update_credit_setting() -> Weight;
	fn add_or_update_credit_data() -> Weight;
	fn burn_for_add_credit() -> Weight;
	fn force_modify_credit_history() -> Weight;
	fn update_nft_class_credit() -> Weight;
	fn update_sum_of_credit_nft_burn_history() -> Weight;
	fn burn_nft() -> Weight;
	fn set_switch_campaign() -> Weight;
	fn set_not_switch_accounts() -> Weight;
	fn set_dpr_price() -> Weight;
}

/// Weights for `pallet_credit` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Credit::CreditSettings` (r:0 w:1)
	/// Proof: `Credit::CreditSettings` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::DailyPocReward` (r:0 w:1)
	/// Proof: `Credit::DailyPocReward` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_credit_setting() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 212_796_000 picoseconds.
		Weight::from_parts(215_682_000, 0)
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `Credit::CreditSettings` (r:1 w:0)
	/// Proof: `Credit::CreditSettings` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::UserCredit` (r:1 w:1)
	/// Proof: `Credit::UserCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_or_update_credit_data() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `831`
		//  Estimated: `4296`
		// Minimum execution time: 347_754_000 picoseconds.
		Weight::from_parts(352_704_000, 4296)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Credit::UserCredit` (r:1 w:1)
	/// Proof: `Credit::UserCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::UserCreditHistory` (r:1 w:1)
	/// Proof: `Credit::UserCreditHistory` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::DPRPerCreditBurned` (r:1 w:0)
	/// Proof: `Credit::DPRPerCreditBurned` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Credit::TotalBurnDPR` (r:1 w:1)
	/// Proof: `Credit::TotalBurnDPR` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::TotalDailyBurnDPR` (r:1 w:1)
	/// Proof: `Credit::TotalDailyBurnDPR` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn burn_for_add_credit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `903`
		//  Estimated: `6196`
		// Minimum execution time: 1_302_449_000 picoseconds.
		Weight::from_parts(1_311_255_000, 6196)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `Credit::UserCreditHistory` (r:1 w:1)
	/// Proof: `Credit::UserCreditHistory` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn force_modify_credit_history() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `262`
		//  Estimated: `3727`
		// Minimum execution time: 301_295_000 picoseconds.
		Weight::from_parts(305_754_000, 3727)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::MiningMachineClassCredit` (r:0 w:1)
	/// Proof: `Credit::MiningMachineClassCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_nft_class_credit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 265_928_000 picoseconds.
		Weight::from_parts(268_773_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::CreditFromBurnNft` (r:0 w:1)
	/// Proof: `Credit::CreditFromBurnNft` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_sum_of_credit_nft_burn_history() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 269_615_000 picoseconds.
		Weight::from_parts(272_421_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Credit::MiningMachineClassCredit` (r:1 w:0)
	/// Proof: `Credit::MiningMachineClassCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::CreditFromBurnNft` (r:1 w:1)
	/// Proof: `Credit::CreditFromBurnNft` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Uniques::Class` (r:1 w:1)
	/// Proof: `Uniques::Class` (`max_values`: None, `max_size`: Some(178), added: 2653, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Asset` (r:1 w:1)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(122), added: 2597, mode: `MaxEncodedLen`)
	/// Storage: `Credit::UserCredit` (r:1 w:1)
	/// Proof: `Credit::UserCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::UserCreditHistory` (r:1 w:0)
	/// Proof: `Credit::UserCreditHistory` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Uniques::Account` (r:0 w:1)
	/// Proof: `Uniques::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::ItemPriceOf` (r:0 w:1)
	/// Proof: `Uniques::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	fn burn_nft() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1066`
		//  Estimated: `4531`
		// Minimum execution time: 1_155_547_000 picoseconds.
		Weight::from_parts(1_167_320_000, 4531)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::CampaignIdSwitch` (r:0 w:3)
	/// Proof: `Credit::CampaignIdSwitch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_switch_campaign() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 227_424_000 picoseconds.
		Weight::from_parts(229_529_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::NotSwitchAccounts` (r:0 w:2)
	/// Proof: `Credit::NotSwitchAccounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_not_switch_accounts() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 210_542_000 picoseconds.
		Weight::from_parts(212_687_000, 3513)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::PriceDiffRate` (r:1 w:0)
	/// Proof: `Credit::PriceDiffRate` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::DprPrice` (r:1 w:0)
	/// Proof: `Credit::DprPrice` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::CurrentPrices` (r:1 w:1)
	/// Proof: `Credit::CurrentPrices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_dpr_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `316`
		//  Estimated: `3513`
		// Minimum execution time: 261_920_000 picoseconds.
		Weight::from_parts(266_048_000, 3513)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Credit::CreditSettings` (r:0 w:1)
	/// Proof: `Credit::CreditSettings` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::DailyPocReward` (r:0 w:1)
	/// Proof: `Credit::DailyPocReward` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_credit_setting() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 212_796_000 picoseconds.
		Weight::from_parts(215_682_000, 0)
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Credit::CreditSettings` (r:1 w:0)
	/// Proof: `Credit::CreditSettings` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::UserCredit` (r:1 w:1)
	/// Proof: `Credit::UserCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn add_or_update_credit_data() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `831`
		//  Estimated: `4296`
		// Minimum execution time: 347_754_000 picoseconds.
		Weight::from_parts(352_704_000, 4296)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Credit::UserCredit` (r:1 w:1)
	/// Proof: `Credit::UserCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::UserCreditHistory` (r:1 w:1)
	/// Proof: `Credit::UserCreditHistory` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::DPRPerCreditBurned` (r:1 w:0)
	/// Proof: `Credit::DPRPerCreditBurned` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Credit::TotalBurnDPR` (r:1 w:1)
	/// Proof: `Credit::TotalBurnDPR` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::TotalDailyBurnDPR` (r:1 w:1)
	/// Proof: `Credit::TotalDailyBurnDPR` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn burn_for_add_credit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `903`
		//  Estimated: `6196`
		// Minimum execution time: 1_302_449_000 picoseconds.
		Weight::from_parts(1_311_255_000, 6196)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	/// Storage: `Credit::UserCreditHistory` (r:1 w:1)
	/// Proof: `Credit::UserCreditHistory` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn force_modify_credit_history() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `262`
		//  Estimated: `3727`
		// Minimum execution time: 301_295_000 picoseconds.
		Weight::from_parts(305_754_000, 3727)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::MiningMachineClassCredit` (r:0 w:1)
	/// Proof: `Credit::MiningMachineClassCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_nft_class_credit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 265_928_000 picoseconds.
		Weight::from_parts(268_773_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::CreditFromBurnNft` (r:0 w:1)
	/// Proof: `Credit::CreditFromBurnNft` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn update_sum_of_credit_nft_burn_history() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 269_615_000 picoseconds.
		Weight::from_parts(272_421_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Credit::MiningMachineClassCredit` (r:1 w:0)
	/// Proof: `Credit::MiningMachineClassCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::CreditFromBurnNft` (r:1 w:1)
	/// Proof: `Credit::CreditFromBurnNft` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Uniques::Class` (r:1 w:1)
	/// Proof: `Uniques::Class` (`max_values`: None, `max_size`: Some(178), added: 2653, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Asset` (r:1 w:1)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(122), added: 2597, mode: `MaxEncodedLen`)
	/// Storage: `Credit::UserCredit` (r:1 w:1)
	/// Proof: `Credit::UserCredit` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::UserCreditHistory` (r:1 w:0)
	/// Proof: `Credit::UserCreditHistory` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Uniques::Account` (r:0 w:1)
	/// Proof: `Uniques::Account` (`max_values`: None, `max_size`: Some(88), added: 2563, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::ItemPriceOf` (r:0 w:1)
	/// Proof: `Uniques::ItemPriceOf` (`max_values`: None, `max_size`: Some(89), added: 2564, mode: `MaxEncodedLen`)
	fn burn_nft() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1066`
		//  Estimated: `4531`
		// Minimum execution time: 1_155_547_000 picoseconds.
		Weight::from_parts(1_167_320_000, 4531)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::CampaignIdSwitch` (r:0 w:3)
	/// Proof: `Credit::CampaignIdSwitch` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_switch_campaign() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 227_424_000 picoseconds.
		Weight::from_parts(229_529_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::NotSwitchAccounts` (r:0 w:2)
	/// Proof: `Credit::NotSwitchAccounts` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_not_switch_accounts() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `150`
		//  Estimated: `3513`
		// Minimum execution time: 210_542_000 picoseconds.
		Weight::from_parts(212_687_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `UserPrivileges::UserPrivileges` (r:1 w:0)
	/// Proof: `UserPrivileges::UserPrivileges` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `Credit::PriceDiffRate` (r:1 w:0)
	/// Proof: `Credit::PriceDiffRate` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::DprPrice` (r:1 w:0)
	/// Proof: `Credit::DprPrice` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Credit::CurrentPrices` (r:1 w:1)
	/// Proof: `Credit::CurrentPrices` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn set_dpr_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `316`
		//  Estimated: `3513`
		// Minimum execution time: 261_920_000 picoseconds.
		Weight::from_parts(266_048_000, 3513)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
