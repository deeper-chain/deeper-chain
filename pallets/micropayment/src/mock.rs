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

use crate as pallet_micropayment;
use crate::testing_utils::*;
use frame_support::{
    pallet_prelude::GenesisBuild,
    parameter_types,
    traits::{OnFinalize, OnInitialize},
    PalletId,
};
use frame_system as system;
use node_primitives::{Balance, Moment};
use sp_core::{crypto::AccountId32, sr25519, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Percent, Permill,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

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
        Micropayment: pallet_micropayment::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
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
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: u64 = 1;
    pub const SpendPeriod: u64 = 24;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const DataDepositPerByte: u64 = 1;
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const MaximumReasonLength: u32 = 16384;
    pub const MaxApprovals: u32 = 100;
}
impl pallet_treasury::Config for Test {
    type PalletId = TreasuryPalletId;
    type Currency = pallet_balances::Pallet<Test>;
    type ApproveOrigin = frame_system::EnsureRoot<AccountId>;
    type RejectOrigin = frame_system::EnsureRoot<AccountId>;
    type Event = Event;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = (); // Just gets burned.
    type WeightInfo = ();
    type SpendFunds = ();
    type MaxApprovals = MaxApprovals;
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
    type WeightInfo = (); //pallet_balances::weights::SubstrateWeight<Test>;
}

type BlockNumber = u64;

const MILLISECS_PER_BLOCK: Moment = 5000;
const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
const CREDIT_ATTENUATION_STEP: u64 = 1;
pub const BLOCKS_PER_ERA: BlockNumber = 6 * EPOCH_DURATION_IN_BLOCKS;

parameter_types! {
    pub const CreditCapTwoEras: u8 = 5;
    pub const CreditAttenuationStep: u64 = CREDIT_ATTENUATION_STEP;
    pub const MinCreditToDelegate: u64 = 100;
    pub const MicropaymentToCreditFactor: u128 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  BLOCKS_PER_ERA;
    pub const DPRPerCreditBurned: Balance = 100;
}

impl pallet_credit::Config for Test {
    type Event = Event;
    type BlocksPerEra = BlocksPerEra;
    type Currency = Balances;
    type CreditCapTwoEras = CreditCapTwoEras;
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
    pub const MinimumPeriod: Moment = 5u64;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
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

impl crate::AccountCreator<AccountId> for TestAccountCreator {
    fn create_account(string: &'static str) -> AccountId {
        get_account_id_from_seed::<sr25519::Public>(string)
    }
}

parameter_types! {
    pub const SecsPerBlock: u32 = 5u32;
    pub const DataPerDPR: u64 = 1024 * 1024 * 1024 * 1024;
    pub const MicropaymentBurn: Percent = Percent::from_percent(10);
}
impl pallet_micropayment::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type CreditInterface = Credit;
    type SecsPerBlock = SecsPerBlock;
    type DataPerDPR = DataPerDPR;
    type AccountCreator = TestAccountCreator;
    type WeightInfo = ();
    type NodeInterface = DeeperNode;
    type MicropaymentBurn = MicropaymentBurn;
    type Slash = Treasury;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (alice(), 500),
            (bob(), 500),
            (charlie(), 500),
            (dave(), 500),
        ],
    }
    .assimilate_storage(&mut storage);
    GenesisBuild::<Test>::assimilate_storage(&pallet_treasury::GenesisConfig, &mut storage)
        .unwrap();

    let ext = sp_io::TestExternalities::from(storage);
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}
