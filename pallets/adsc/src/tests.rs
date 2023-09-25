// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Macro for creating the tests for the module.

use frame_support::traits::fungible::Mutate as TokenMutate;
use frame_support::traits::fungibles::Mutate;
use frame_support::traits::{
    nonfungibles::Inspect, AsEnsureOriginWithArg, ConstU128, ConstU32, Hooks,
};
use frame_support::{assert_noop, assert_ok, parameter_types, weights::Weight, PalletId};
use pallet_user_privileges::H160;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use super::*;
use crate::{self as pallet_adsc};
use node_primitives::{
    user_privileges::{Privilege, UserPrivilegeInterface},
    Moment, DPR,
};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Adsc: pallet_adsc::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Config<T>, Event<T>},
        Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type BlockNumber = u64;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MinimumBurnedDPR: u64 = 1;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: Moment = 5u64;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = u128;
    type AssetId = u32;
    type AssetIdParameter = u32;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type AssetDeposit = ConstU128<1>;
    type AssetAccountDeposit = ConstU128<10>;
    type MetadataDepositBase = ConstU128<1>;
    type MetadataDepositPerByte = ConstU128<1>;
    type ApprovalDeposit = ConstU128<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
    type RemoveItemsLimit = ConstU32<5>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

parameter_types! {
    pub const KeyLimit: u32 = 32;
    pub const ValueLimit: u32 = 256;
    pub const StringLimit: u32 = 50;
}

impl pallet_uniques::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = u32;
    type ItemId = u32;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<u64>;
    type CollectionDeposit = ConstU128<0>;
    type ItemDeposit = ConstU128<0>;
    type MetadataDepositBase = ConstU128<0>;
    type AttributeDepositBase = ConstU128<0>;
    type DepositPerByte = ConstU128<0>;
    type StringLimit = ConstU32<50>;
    type KeyLimit = ConstU32<50>;
    type ValueLimit = ConstU32<50>;
    type WeightInfo = ();
    type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<u64>>;
    type Locker = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 5000;
pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: u64 = 60 / SECS_PER_BLOCK;
pub const BLOCKS_PER_ERA: u64 = (1 * EPOCH_DURATION_IN_BLOCKS) as u64;

parameter_types! {
    pub const AdscPalletId: PalletId = PalletId(*b"dep/adsc");
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;

    type AdscCurrency = Assets;
    type DprCurrency = Balances;
    type WeightInfo = ();
    type Time = Timestamp;
    type AdscId = ConstU32<1>;
    type PalletId = AdscPalletId;
    type UserPrivilegeInterface = U128FakeUserPrivilege;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        // Total issuance will be 200 with treasury account initialized at ED.
        balances: vec![(0, 100), (1, 98), (2, 1)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        Timestamp::set_timestamp(System::block_number() * 1440 * 5000);
        Adsc::on_initialize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}

#[test]
fn adsc_pay_reward() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 0, true, 10));

        // start day is day 0
        CurrentAdscBaseReward::<Test>::put(1560 * DPR);
        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 2));

        // 8,9 only check when 365
        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 8));
        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 9));

        run_to_block(BLOCKS_PER_ERA + 3);
        assert_eq!(Assets::balance(1, &2), 1560 * DPR);

        assert_eq!(CurrentMintedAdsc::<Test>::get(), 1560 * 3 * DPR);
        assert_eq!(Assets::total_supply(1), 1560 * 3 * DPR);

        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 3));

        run_to_block(2 * BLOCKS_PER_ERA + 3);
        assert_eq!(Assets::balance(1, &2), 3115726025880000000000);
        assert_eq!(Assets::balance(1, &3), 1560 * DPR);

        run_to_block(365 * BLOCKS_PER_ERA + 3);
        assert_eq!(Assets::balance(1, &2), 285479999719200000000000);
        assert_eq!(Assets::balance(1, &3), 285475725746640000000000);

        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 3));

        assert_eq!(AdscStakers::<Test>::get(3), Some(365));
        assert_eq!(AdscStakers::<Test>::get(2), Some(0));

        run_to_block(366 * BLOCKS_PER_ERA + 3);
        assert_eq!(
            Assets::balance(1, &3),
            285475725746640000000000 + 1560 * DPR
        );

        assert_eq!(AdscStakers::<Test>::get(2), None);
        assert_eq!(Assets::balance(1, &2), 285479999719200000000000);
        assert_eq!(Assets::balance(1, &8), 285479999719200000000000);
        assert_eq!(Assets::balance(1, &9), 285479999719200000000000);
    });
}

#[test]
fn adsc_half_reward() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 0, true, 10));

        CurrentAdscBaseReward::<Test>::put(1560 * DPR);

        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 2));
        assert_ok!(Adsc::add_adsc_staking_account(RuntimeOrigin::signed(1), 3));

        CurrentHalfTarget::<Test>::put(1560 * 2 * DPR);

        run_to_block(BLOCKS_PER_ERA + 3);
        assert_eq!(Assets::balance(1, &2), 1560 * DPR);
        assert_eq!(Assets::balance(1, &3), 1560 * DPR);

        assert_eq!(
            CurrentHalfTarget::<Test>::get(),
            1560 * 2 * DPR + 10_000_000_000 * DPR
        );
        //half base reward
        assert_eq!(CurrentAdscBaseReward::<Test>::get(), 1560 / 2 * DPR);
        CurrentHalfTarget::<Test>::put(1560 * 3 * DPR);

        run_to_block(2 * BLOCKS_PER_ERA + 3);
        // added balance = 1560/2 * (364/365)*DPR
        assert_eq!(Assets::balance(1, &2), 2337863012940000000000);
        assert_eq!(Assets::balance(1, &3), 2337863012940000000000);

        run_to_block(3 * BLOCKS_PER_ERA + 3);
        assert_eq!(CurrentAdscBaseReward::<Test>::get(), 1560 / 2 / 2 * DPR);
    });
}

#[test]
fn adsc_add_nft() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::create(RuntimeOrigin::signed(1), 1, 1, 1));

        assert_ok!(Uniques::create(RuntimeOrigin::signed(1), 1, 1));

        assert_ok!(Adsc::add_adsc_staking_account_with_nft(
            RuntimeOrigin::signed(1),
            2,
            1,
            1,
            b"aa".to_vec()
        ));

        assert_eq!(Uniques::owner(1, 1), Some(2));
        assert_eq!(Uniques::attribute(&1, &1, &[]), Some(b"aa".to_vec()));

        assert_ok!(Adsc::add_adsc_staking_account_with_nft(
            RuntimeOrigin::signed(1),
            2,
            1,
            2,
            b"other_aa".to_vec()
        ));

        assert_eq!(Uniques::owner(1, 1), None);
        assert_eq!(Uniques::attribute(&1, &1, &[]), None);

        assert_eq!(Uniques::owner(1, 2), Some(2));
        assert_eq!(Uniques::attribute(&1, &2, &[]), Some(b"other_aa".to_vec()));
    });
}

#[test]
fn swap_adsc() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 0, true, 10));

        assert_ok!(Assets::mint_into(1, &3, 2 * DPR));
        assert_ok!(Balances::set_balance(
            RuntimeOrigin::root(),
            Adsc::account_id(),
            100 * DPR,
            0
        ));
        assert_ok!(Adsc::swap_adsc_to_dpr(RuntimeOrigin::signed(3), 2 * DPR));
        assert_eq!(Balances::free_balance(&3), DPR);

        AdscExchangeRate::<Test>::put((6, 4));
        assert_ok!(Assets::mint_into(1, &4, 3 * DPR));
        assert_ok!(Adsc::swap_adsc_to_dpr(RuntimeOrigin::signed(4), 3 * DPR));
        assert_eq!(Balances::free_balance(&4), 2 * DPR);

        AdscExchangeRate::<Test>::put((3, 7));
        assert_ok!(Assets::mint_into(1, &5, 3 * DPR));
        assert_ok!(Adsc::swap_adsc_to_dpr(RuntimeOrigin::signed(5), 3 * DPR));
        assert_eq!(Balances::free_balance(&5), 7 * DPR);
    });
}

#[test]
fn swap_dpr() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 0, true, 10));
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), 3, DPR, 0));
        assert_ok!(Assets::mint_into(1, &Adsc::account_id(), 2 * DPR));

        assert_ok!(Adsc::swap_dpr_to_adsc(RuntimeOrigin::signed(3), DPR));
        assert_eq!(Assets::balance(1, &3), 2 * DPR);
    });
}

#[test]
fn swap_dpr_pool_not_enough() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 0, true, 10));
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), 3, DPR, 0));

        assert_ok!(Adsc::do_add_pool_dpr_adsc(0, DPR));
        assert_noop!(
            Adsc::swap_dpr_to_adsc(RuntimeOrigin::signed(3), DPR),
            pallet_assets::Error::<Test>::BalanceLow
        );
        assert_ok!(Adsc::do_add_pool_dpr_adsc(0, DPR));
        assert_ok!(Adsc::swap_dpr_to_adsc(RuntimeOrigin::signed(3), DPR));

        assert_ok!(Balances::burn_from(&Adsc::account_id(), DPR));

        assert_noop!(
            Adsc::swap_adsc_to_dpr(RuntimeOrigin::signed(3), 2 * DPR),
            pallet_balances::Error::<Test>::InsufficientBalance
        );
        assert_ok!(Adsc::do_add_pool_dpr_adsc(DPR, 0));
        assert_ok!(Adsc::swap_adsc_to_dpr(RuntimeOrigin::signed(3), 2 * DPR));

        assert_eq!(TotalPoolAdsc::<Test>::get(), 2 * DPR);
        assert_eq!(TotalPoolDpr::<Test>::get(), DPR);
    });
}

pub struct U128FakeUserPrivilege;

impl UserPrivilegeInterface<u64> for U128FakeUserPrivilege {
    fn has_privilege(user: &u64, _p: Privilege) -> bool {
        if user == &1 {
            return true;
        }
        false
    }

    fn has_evm_privilege(_user: &H160, _p: Privilege) -> bool {
        true
    }
}
