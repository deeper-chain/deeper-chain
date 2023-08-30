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

use frame_support::traits::{fungibles::Mutate, AsEnsureOriginWithArg, ConstU32, ConstU64, Hooks};
use frame_support::{assert_ok, parameter_types, weights::Weight, PalletId};
use pallet_user_privileges::H160;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use super::*;
use crate::{self as pallet_adst};
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
        Adst: pallet_adst::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Config<T>, Event<T>},
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
    type AccountData = pallet_balances::AccountData<u64>;
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
    type Balance = u64;
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
    type AssetDeposit = ConstU64<1>;
    type AssetAccountDeposit = ConstU64<10>;
    type MetadataDepositBase = ConstU64<1>;
    type MetadataDepositPerByte = ConstU64<1>;
    type ApprovalDeposit = ConstU64<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
    type RemoveItemsLimit = ConstU32<5>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub const MILLISECS_PER_BLOCK: u64 = 5000;
pub const SECS_PER_BLOCK: u64 = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: u64 = 60 / SECS_PER_BLOCK;
pub const BlocksPerEra: u64 = (1 * EPOCH_DURATION_IN_BLOCKS) as u64;

parameter_types! {
    pub const AdstPalletId: PalletId = PalletId(*b"dep/adst");
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;

    type AdstCurrency = Assets;
    type WeightInfo = ();
    type Time = Timestamp;
    type AdstId = ConstU32<1>;
    type PalletId = AdstPalletId;
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

        println!("Timestamp::now() {}", Timestamp::now());
        Adst::on_initialize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}

#[test]
fn adst_pay_reward() {
    new_test_ext().execute_with(|| {
        Adst::on_runtime_upgrade();

        assert_ok!(<Test as Config>::AdstCurrency::mint_into(1, &11, 1 * DPR));
        assert_eq!(Assets::balance(1, &11), 1 * DPR);
        // start day is day 0

        assert_ok!(Adst::add_adst_staking_account(RuntimeOrigin::signed(1), 2));

        // 8,9 only check when 180
        assert_ok!(Adst::add_adst_staking_account(RuntimeOrigin::signed(1), 8));
        assert_ok!(Adst::add_adst_staking_account(RuntimeOrigin::signed(1), 9));

        run_to_block(BlocksPerEra + 3);
        assert_eq!(Assets::balance(1, &2), 1560 * DPR);

        assert_ok!(Adst::add_adst_staking_account(RuntimeOrigin::signed(1), 3));

        run_to_block(2 * BlocksPerEra + 3);
        assert_eq!(Assets::balance(1, &2), 3111333332640000000000);
        assert_eq!(Assets::balance(1, &3), 1560 * DPR);

        run_to_block(180 * BlocksPerEra + 3);
        assert_eq!(Assets::balance(1, &2), 141179999875200000000000);
        assert_eq!(Assets::balance(1, &3), 141171333209400000000000);

        assert_ok!(Adst::add_adst_staking_account(RuntimeOrigin::signed(1), 3));

        assert_eq!(AdstStakers::<Test>::get(3), Some(180));
        assert_eq!(AdstStakers::<Test>::get(2), Some(0));

        run_to_block(181 * BlocksPerEra + 3);
        assert_eq!(
            Assets::balance(1, &3),
            141171333209400000000000 + 1560 * DPR
        );

        assert_eq!(AdstStakers::<Test>::get(2), None);
        assert_eq!(Assets::balance(1, &2), 141179999875200000000000);
        assert_eq!(Assets::balance(1, &8), 141179999875200000000000);
        assert_eq!(Assets::balance(1, &9), 141179999875200000000000);
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
