// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for pallet_staking
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-15, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/deeper-chain
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_staking
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./pallets/staking/src/weights.rs
// --template=./.maintain/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_staking.
pub trait WeightInfo {
    fn bond() -> Weight;
    fn bond_extra() -> Weight;
    fn unbond() -> Weight;
    fn withdraw_unbonded_update(s: u32) -> Weight;
    fn withdraw_unbonded_kill(s: u32) -> Weight;
    fn validate() -> Weight;
    fn delegate(n: u32) -> Weight;
    fn undelegate() -> Weight;
    fn staking_delegate() -> Weight;
    fn chill() -> Weight;
    fn set_payee() -> Weight;
    fn set_controller() -> Weight;
    fn set_era_validator_reward() -> Weight;
    fn set_validator_count() -> Weight;
    fn increase_validator_count(n: u32) -> Weight;
    fn scale_validator_count(n: u32) -> Weight;
    fn force_no_eras() -> Weight;
    fn force_new_era() -> Weight;
    fn force_new_era_always() -> Weight;
    fn set_invulnerables(v: u32) -> Weight;
    fn set_validator_whitelist(v: u32) -> Weight;
    fn force_unstake(s: u32) -> Weight;
    fn increase_mining_reward(r: u32) -> Weight;
    fn cancel_deferred_slash(s: u32) -> Weight;
    fn rebond(l: u32) -> Weight;
    fn set_history_depth(e: u32) -> Weight;
    fn reap_stash(s: u32) -> Weight;
    fn new_era(v: u32, d: u32) -> Weight;
}

/// Weights for pallet_staking using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bond() -> Weight {
        (46_185_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    fn bond_extra() -> Weight {
        (39_689_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn unbond() -> Weight {
        (36_377_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn withdraw_unbonded_update(s: u32) -> Weight {
        (37_516_000 as Weight) // Standard Error: 0
            .saturating_add((66_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn withdraw_unbonded_kill(s: u32) -> Weight {
        (54_182_000 as Weight) // Standard Error: 1_000
            .saturating_add((1_038_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn validate() -> Weight {
        (10_501_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn delegate(_n: u32) -> Weight {
        (46_255_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn undelegate() -> Weight {
        (37_300_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    fn staking_delegate() -> Weight {
        (108_056_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(11 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }

    fn chill() -> Weight {
        (10_500_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_payee() -> Weight {
        (8_301_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_controller() -> Weight {
        (17_772_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn set_era_validator_reward() -> Weight {
        (1_403_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_validator_count() -> Weight {
        (1_202_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn increase_validator_count(_n: u32) -> Weight {
        (3_677_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn scale_validator_count(n: u32) -> Weight {
        (3_514_000 as Weight) // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn force_no_eras() -> Weight {
        (1_309_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn force_new_era() -> Weight {
        (1_281_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn force_new_era_always() -> Weight {
        (1_315_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_invulnerables(v: u32) -> Weight {
        (1_532_000 as Weight) // Standard Error: 0
            .saturating_add((65_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn set_validator_whitelist(v: u32) -> Weight {
        (1_501_000 as Weight) // Standard Error: 0
            .saturating_add((65_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn force_unstake(s: u32) -> Weight {
        (33_823_000 as Weight) // Standard Error: 1_000
            .saturating_add((1_033_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn increase_mining_reward(_r: u32) -> Weight {
        (3_584_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn cancel_deferred_slash(s: u32) -> Weight {
        (1_071_873_000 as Weight) // Standard Error: 60_000
            .saturating_add((5_404_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn rebond(l: u32) -> Weight {
        (24_197_000 as Weight) // Standard Error: 1_000
            .saturating_add((49_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn set_history_depth(e: u32) -> Weight {
        (0 as Weight) // Standard Error: 37_000
            .saturating_add((16_254_000 as Weight).saturating_mul(e as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((6 as Weight).saturating_mul(e as Weight)))
    }
    fn reap_stash(s: u32) -> Weight {
        (35_683_000 as Weight) // Standard Error: 0
            .saturating_add((1_017_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn new_era(v: u32, d: u32) -> Weight {
        (39_945_000 as Weight) // Standard Error: 352_000
            .saturating_add((27_967_000 as Weight).saturating_mul(v as Weight)) // Standard Error: 2_000
            .saturating_add((352_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((4 as Weight).saturating_mul(v as Weight)))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(v as Weight)))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn bond() -> Weight {
        (46_185_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }
    fn bond_extra() -> Weight {
        (39_689_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    fn unbond() -> Weight {
        (36_377_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn withdraw_unbonded_update(s: u32) -> Weight {
        (37_516_000 as Weight) // Standard Error: 0
            .saturating_add((66_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn withdraw_unbonded_kill(s: u32) -> Weight {
        (54_182_000 as Weight) // Standard Error: 1_000
            .saturating_add((1_038_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn validate() -> Weight {
        (10_501_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn delegate(_n: u32) -> Weight {
        (46_255_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(8 as Weight))
            .saturating_add(RocksDbWeight::get().writes(5 as Weight))
    }
    fn undelegate() -> Weight {
        (37_300_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(5 as Weight))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
    }

    fn staking_delegate() -> Weight {
        (108_056_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(11 as Weight))
            .saturating_add(RocksDbWeight::get().writes(8 as Weight))
    }

    fn chill() -> Weight {
        (10_500_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_payee() -> Weight {
        (8_301_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_controller() -> Weight {
        (17_772_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn set_era_validator_reward() -> Weight {
        (1_403_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_validator_count() -> Weight {
        (1_202_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn increase_validator_count(_n: u32) -> Weight {
        (3_677_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn scale_validator_count(n: u32) -> Weight {
        (3_514_000 as Weight) // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn force_no_eras() -> Weight {
        (1_309_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn force_new_era() -> Weight {
        (1_281_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn force_new_era_always() -> Weight {
        (1_315_000 as Weight).saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_invulnerables(v: u32) -> Weight {
        (1_532_000 as Weight) // Standard Error: 0
            .saturating_add((65_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn set_validator_whitelist(v: u32) -> Weight {
        (1_501_000 as Weight) // Standard Error: 0
            .saturating_add((65_000 as Weight).saturating_mul(v as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn force_unstake(s: u32) -> Weight {
        (33_823_000 as Weight) // Standard Error: 1_000
            .saturating_add((1_033_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn increase_mining_reward(_r: u32) -> Weight {
        (3_584_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn cancel_deferred_slash(s: u32) -> Weight {
        (1_071_873_000 as Weight) // Standard Error: 60_000
            .saturating_add((5_404_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    fn rebond(l: u32) -> Weight {
        (24_197_000 as Weight) // Standard Error: 1_000
            .saturating_add((49_000 as Weight).saturating_mul(l as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(3 as Weight))
    }
    fn set_history_depth(e: u32) -> Weight {
        (0 as Weight) // Standard Error: 37_000
            .saturating_add((16_254_000 as Weight).saturating_mul(e as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes((6 as Weight).saturating_mul(e as Weight)))
    }
    fn reap_stash(s: u32) -> Weight {
        (35_683_000 as Weight) // Standard Error: 0
            .saturating_add((1_017_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    fn new_era(v: u32, d: u32) -> Weight {
        (39_945_000 as Weight) // Standard Error: 352_000
            .saturating_add((27_967_000 as Weight).saturating_mul(v as Weight)) // Standard Error: 2_000
            .saturating_add((352_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().reads((4 as Weight).saturating_mul(v as Weight)))
            .saturating_add(RocksDbWeight::get().writes(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes((2 as Weight).saturating_mul(v as Weight)))
    }
}
