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
//! DATE: 2022-08-05, STEPS: 50, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/deeper-chain
// benchmark
// pallet
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
    fn staking_delegate() -> Weight;
    fn usdt_staking_delegate() -> Weight;
    fn delegate(n: u32) -> Weight;
    fn undelegate() -> Weight;
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
    fn npow_mint() -> Weight;
}

/// Weights for pallet_staking using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bond() -> Weight {
        Weight::from_ref_time(46_364_000 as u64)
            .saturating_add(T::DbWeight::get().reads(5 as u64))
            .saturating_add(T::DbWeight::get().writes(4 as u64))
    }
    fn bond_extra() -> Weight {
        Weight::from_ref_time(40_194_000 as u64)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    fn unbond() -> Weight {
        Weight::from_ref_time(36_389_000 as u64)
            .saturating_add(T::DbWeight::get().reads(5 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
    }
    fn withdraw_unbonded_update(s: u32) -> Weight {
        Weight::from_ref_time(37_944_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (68_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(5 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
    }
    fn withdraw_unbonded_kill(s: u32) -> Weight {
        Weight::from_ref_time(54_374_000 as u64) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (1_072_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(7 as u64))
            .saturating_add(T::DbWeight::get().writes(7 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(s as u64)))
    }
    fn validate() -> Weight {
        Weight::from_ref_time(10_956_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn staking_delegate() -> Weight {
        Weight::from_ref_time(176_198_000 as u64)
            .saturating_add(T::DbWeight::get().reads(15 as u64))
            .saturating_add(T::DbWeight::get().writes(9 as u64))
    }
    fn delegate(_n: u32) -> Weight {
        Weight::from_ref_time(45_723_000 as u64)
            .saturating_add(T::DbWeight::get().reads(8 as u64))
            .saturating_add(T::DbWeight::get().writes(5 as u64))
    }
    fn undelegate() -> Weight {
        Weight::from_ref_time(37_470_000 as u64)
            .saturating_add(T::DbWeight::get().reads(5 as u64))
            .saturating_add(T::DbWeight::get().writes(4 as u64))
    }
    fn chill() -> Weight {
        Weight::from_ref_time(10_303_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn set_payee() -> Weight {
        Weight::from_ref_time(8_128_000 as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn set_controller() -> Weight {
        Weight::from_ref_time(17_292_000 as u64)
            .saturating_add(T::DbWeight::get().reads(3 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
    }
    fn set_era_validator_reward() -> Weight {
        Weight::from_ref_time(1_364_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn set_validator_count() -> Weight {
        Weight::from_ref_time(1_236_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn increase_validator_count(_n: u32) -> Weight {
        Weight::from_ref_time(3_722_000 as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn scale_validator_count(n: u32) -> Weight {
        Weight::from_ref_time(3_515_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (2_000 as u64).saturating_mul(n as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn force_no_eras() -> Weight {
        Weight::from_ref_time(1_360_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn force_new_era() -> Weight {
        Weight::from_ref_time(1_357_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn force_new_era_always() -> Weight {
        Weight::from_ref_time(1_375_000 as u64).saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn set_invulnerables(v: u32) -> Weight {
        Weight::from_ref_time(1_411_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (11_000 as u64).saturating_mul(v as u64),
            ))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn set_validator_whitelist(v: u32) -> Weight {
        Weight::from_ref_time(11_389_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (26_000 as u64).saturating_mul(v as u64),
            ))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn force_unstake(s: u32) -> Weight {
        Weight::from_ref_time(34_222_000 as u64) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (1_060_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(7 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(s as u64)))
    }
    fn increase_mining_reward(_r: u32) -> Weight {
        Weight::from_ref_time(12_185_000 as u64)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn cancel_deferred_slash(s: u32) -> Weight {
        Weight::from_ref_time(1_122_877_000 as u64) // Standard Error: 69_000
            .saturating_add(Weight::from_ref_time(
                (6_194_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    fn rebond(l: u32) -> Weight {
        Weight::from_ref_time(24_117_000 as u64) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (41_000 as u64).saturating_mul(l as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
    }
    fn set_history_depth(e: u32) -> Weight {
        Weight::from_ref_time(0 as u64) // Standard Error: 37_000
            .saturating_add(Weight::from_ref_time(
                (16_583_000 as u64).saturating_mul(e as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
            .saturating_add(T::DbWeight::get().writes((6 as u64).saturating_mul(e as u64)))
    }
    fn reap_stash(s: u32) -> Weight {
        Weight::from_ref_time(35_231_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (1_054_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(7 as u64))
            .saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(s as u64)))
    }
    fn new_era(v: u32, d: u32) -> Weight {
        Weight::from_ref_time(37_804_000 as u64) // Standard Error: 292_000
            .saturating_add(Weight::from_ref_time(
                (28_064_000 as u64).saturating_mul(v as u64),
            )) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (218_000 as u64).saturating_mul(d as u64),
            ))
            .saturating_add(T::DbWeight::get().reads(6 as u64))
            .saturating_add(T::DbWeight::get().reads((4 as u64).saturating_mul(v as u64)))
            .saturating_add(T::DbWeight::get().writes(4 as u64))
            .saturating_add(T::DbWeight::get().writes((2 as u64).saturating_mul(v as u64)))
    }
    fn usdt_staking_delegate() -> Weight {
        Weight::from_ref_time(105_590_000 as u64)
            .saturating_add(T::DbWeight::get().reads(15 as u64))
            .saturating_add(T::DbWeight::get().writes(8 as u64))
    }
    fn npow_mint() -> Weight {
        Weight::from_ref_time(33_965_000 as u64)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn bond() -> Weight {
        Weight::from_ref_time(46_364_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }
    fn bond_extra() -> Weight {
        Weight::from_ref_time(40_194_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
    }
    fn unbond() -> Weight {
        Weight::from_ref_time(36_389_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
    fn withdraw_unbonded_update(s: u32) -> Weight {
        Weight::from_ref_time(37_944_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (68_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
    fn withdraw_unbonded_kill(s: u32) -> Weight {
        Weight::from_ref_time(54_374_000 as u64) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (1_072_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(7 as u64))
            .saturating_add(RocksDbWeight::get().writes(7 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(s as u64)))
    }
    fn validate() -> Weight {
        Weight::from_ref_time(10_956_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn staking_delegate() -> Weight {
        Weight::from_ref_time(176_198_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(15 as u64))
            .saturating_add(RocksDbWeight::get().writes(9 as u64))
    }
    fn delegate(_n: u32) -> Weight {
        Weight::from_ref_time(45_723_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(8 as u64))
            .saturating_add(RocksDbWeight::get().writes(5 as u64))
    }
    fn undelegate() -> Weight {
        Weight::from_ref_time(37_470_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(5 as u64))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
    }
    fn chill() -> Weight {
        Weight::from_ref_time(10_303_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn set_payee() -> Weight {
        Weight::from_ref_time(8_128_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn set_controller() -> Weight {
        Weight::from_ref_time(17_292_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(3 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
    fn set_era_validator_reward() -> Weight {
        Weight::from_ref_time(1_364_000 as u64)
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn set_validator_count() -> Weight {
        Weight::from_ref_time(1_236_000 as u64)
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn increase_validator_count(_n: u32) -> Weight {
        Weight::from_ref_time(3_722_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn scale_validator_count(n: u32) -> Weight {
        Weight::from_ref_time(3_515_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (2_000 as u64).saturating_mul(n as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn force_no_eras() -> Weight {
        Weight::from_ref_time(1_360_000 as u64)
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn force_new_era() -> Weight {
        Weight::from_ref_time(1_357_000 as u64)
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn force_new_era_always() -> Weight {
        Weight::from_ref_time(1_375_000 as u64)
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn set_invulnerables(v: u32) -> Weight {
        Weight::from_ref_time(1_411_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (11_000 as u64).saturating_mul(v as u64),
            ))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn set_validator_whitelist(v: u32) -> Weight {
        Weight::from_ref_time(11_389_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (26_000 as u64).saturating_mul(v as u64),
            ))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn force_unstake(s: u32) -> Weight {
        Weight::from_ref_time(34_222_000 as u64) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (1_060_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(7 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(s as u64)))
    }
    fn increase_mining_reward(_r: u32) -> Weight {
        Weight::from_ref_time(12_185_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn cancel_deferred_slash(s: u32) -> Weight {
        Weight::from_ref_time(1_122_877_000 as u64) // Standard Error: 69_000
            .saturating_add(Weight::from_ref_time(
                (6_194_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(1 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
    fn rebond(l: u32) -> Weight {
        Weight::from_ref_time(24_117_000 as u64) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (41_000 as u64).saturating_mul(l as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(3 as u64))
    }
    fn set_history_depth(e: u32) -> Weight {
        Weight::from_ref_time(0 as u64) // Standard Error: 37_000
            .saturating_add(Weight::from_ref_time(
                (16_583_000 as u64).saturating_mul(e as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(2 as u64))
            .saturating_add(RocksDbWeight::get().writes((6 as u64).saturating_mul(e as u64)))
    }
    fn reap_stash(s: u32) -> Weight {
        Weight::from_ref_time(35_231_000 as u64) // Standard Error: 0
            .saturating_add(Weight::from_ref_time(
                (1_054_000 as u64).saturating_mul(s as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(4 as u64))
            .saturating_add(RocksDbWeight::get().writes(7 as u64))
            .saturating_add(RocksDbWeight::get().writes((1 as u64).saturating_mul(s as u64)))
    }
    fn new_era(v: u32, d: u32) -> Weight {
        Weight::from_ref_time(37_804_000 as u64) // Standard Error: 292_000
            .saturating_add(Weight::from_ref_time(
                (28_064_000 as u64).saturating_mul(v as u64),
            )) // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(
                (218_000 as u64).saturating_mul(d as u64),
            ))
            .saturating_add(RocksDbWeight::get().reads(6 as u64))
            .saturating_add(RocksDbWeight::get().reads((4 as u64).saturating_mul(v as u64)))
            .saturating_add(RocksDbWeight::get().writes(4 as u64))
            .saturating_add(RocksDbWeight::get().writes((2 as u64).saturating_mul(v as u64)))
    }
    fn usdt_staking_delegate() -> Weight {
        Weight::from_ref_time(105_590_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(15 as u64))
            .saturating_add(RocksDbWeight::get().writes(8 as u64))
    }
    fn npow_mint() -> Weight {
        Weight::from_ref_time(33_965_000 as u64)
            .saturating_add(RocksDbWeight::get().reads(2 as u64))
            .saturating_add(RocksDbWeight::get().writes(1 as u64))
    }
}
