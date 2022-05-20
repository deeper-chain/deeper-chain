// Copyright (C) 2021 Deeper Network Inc.
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

use crate as pallet_credit_accumulation;
use crate::testing_utils::*;
use frame_support::parameter_types;
use frame_support::traits::{ConstU128, ConstU32, OnFinalize, OnInitialize};

use frame_system as system;
use node_primitives::{Balance, Moment};
use pallet_credit::{CreditData, CreditLevel};
use pallet_micropayment::AccountCreator;
use sp_core::testing::SR25519;
use sp_core::{crypto::AccountId32, sr25519, H256};
use sp_keystore::SyncCryptoStore;
use sp_keystore::{testing::KeyStore, KeystoreExt};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use std::sync::Arc;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 5000;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Event<T>, Config<T>},
        Credit: pallet_credit::{Pallet, Call, Storage, Event<T>, Config<T>},
        DeeperNode: pallet_deeper_node::{Pallet, Call, Storage, Event<T>, Config<T>},
        CreditAccumulation: pallet_credit_accumulation::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

type AccountId = AccountId32;

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 100;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = (); //pallet_balances::weights::SubstrateWeight<Test>;
}

type BlockNumber = u64;

const MILLISECS_PER_BLOCK: Moment = 5000;
const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
const CREDIT_ATTENUATION_STEP: u64 = 1;
pub const BLOCKS_PER_ERA: BlockNumber = 6 * EPOCH_DURATION_IN_BLOCKS;

parameter_types! {
    pub const CreditAttenuationStep: u64 = CREDIT_ATTENUATION_STEP;
    pub const MinCreditToDelegate: u64 = 100;
    pub const MicropaymentToCreditFactor: u128 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  BLOCKS_PER_ERA;
}

parameter_types! {
    pub const MinimumPeriod: Moment = 5u64;
    pub const DPRPerCreditBurned: Balance = 100;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

const MILLICENTS: Balance = 10_000_000_000_000;
const CENTS: Balance = 1_000 * MILLICENTS;
const DOLLARS: Balance = 100 * CENTS;
parameter_types! {
    pub const ClassDeposit: Balance = 100 * DOLLARS;
    pub const InstanceDeposit: Balance = 1 * DOLLARS;
    pub const KeyLimit: u32 = 32;
    pub const ValueLimit: u32 = 256;
    pub const StringLimit: u32 = 50;
}

impl pallet_uniques::Config for Test {
    type Event = Event;
    type ClassId = u32;
    type InstanceId = u32;
    type Currency = Balances;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type ClassDeposit = ConstU128<2>;
    type InstanceDeposit = ConstU128<1>;
    type MetadataDepositBase = ConstU128<1>;
    type AttributeDepositBase = ConstU128<1>;
    type DepositPerByte = ConstU128<1>;
    type StringLimit = ConstU32<50>;
    type KeyLimit = ConstU32<50>;
    type ValueLimit = ConstU32<50>;
    type WeightInfo = ();
}

impl pallet_credit::Config for Test {
    type Event = Event;
    type BlocksPerEra = BlocksPerEra;
    type Currency = Balances;
    type CreditAttenuationStep = CreditAttenuationStep;
    type MinCreditToDelegate = MinCreditToDelegate;
    type MicropaymentToCreditFactor = MicropaymentToCreditFactor;
    type NodeInterface = DeeperNode;
    type WeightInfo = ();
    type UnixTime = Timestamp;
    type SecsPerBlock = SecsPerBlock;
    type DPRPerCreditBurned = DPRPerCreditBurned;
    type BurnedTo = ();
}

parameter_types! {
    pub const MinLockAmt: u32 = 100;
    pub const MaxDurationEras: u8 = 7;
    pub const MaxIpLength: usize = 256;
}
impl pallet_deeper_node::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinLockAmt = MinLockAmt;
    type MaxDurationEras = MaxDurationEras;
    type BlocksPerEra = BlocksPerEra;
    type MaxIpLength = MaxIpLength;
    type WeightInfo = ();
}

pub struct TestAccountCreator;

impl AccountCreator<AccountId> for TestAccountCreator {
    fn create_account(string: &'static str) -> AccountId {
        get_account_id_from_seed::<sr25519::Public>(string)
    }
}

parameter_types! {
    pub const SecsPerBlock: u32 = 5u32;
    pub const DataPerDPR: u64 = 1024 * 1024 * 1024 * 1024;
}
impl pallet_credit_accumulation::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type CreditInterface = Credit;
    type AccountCreator = TestAccountCreator;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![(alice(), 500), (bob(), 500)],
    }
    .assimilate_storage(&mut storage);

    let _ = pallet_credit::GenesisConfig::<Test> {
        credit_settings: vec![],
        user_credit_data: vec![
            (
                alice(),
                CreditData {
                    campaign_id: 0,
                    credit: 210,
                    initial_credit_level: CreditLevel::Two,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 0,
                    current_credit_level: CreditLevel::Two,
                    reward_eras: 1,
                },
            ),
            (
                bob(),
                CreditData {
                    campaign_id: 0,
                    credit: 220,
                    initial_credit_level: CreditLevel::Two,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 0,
                    current_credit_level: CreditLevel::Two,
                    reward_eras: 1,
                },
            ),
        ],
    }
    .assimilate_storage(&mut storage);

    let mut ext = sp_io::TestExternalities::from(storage);
    // initialize test keystore, we can access this key with
    // sp_io::crypto::sr25519_public_keys(SR25519)[0];
    let keystore = KeyStore::new();
    let _ = keystore
        .sr25519_generate_new(SR25519, Some("//Bob"))
        .unwrap();
    ext.register_extension(KeystoreExt(Arc::new(keystore)));
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Timestamp::set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);
    }
}
