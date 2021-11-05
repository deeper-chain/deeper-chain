use crate as pallet_eth_sub_bridge;
use frame_support::{
    parameter_types,
    traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use node_primitives::{Balance, BlockNumber, Moment};

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
        Bridge: pallet_eth_sub_bridge::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

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
    type AccountId = u64;
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
    pub const ExistentialDeposit: Balance = 500;
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

pub const MILLISECS_PER_BLOCK: Moment = 5000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
parameter_types! {
    pub const MinimumPeriod: Moment = 2500u64;
    pub const BlocksPerEra: BlockNumber =  6 * EPOCH_DURATION_IN_BLOCKS;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_eth_sub_bridge::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type BlocksPerEra = BlocksPerEra;
    type WeightInfo = ();
}

const V1: u64 = 1;
const V2: u64 = 2;
const V3: u64 = 3;
const USER1: u64 = 5;
const USER2: u64 = 6;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (V1, 100000),
            (V2, 100000),
            (V3, 100000),
            (USER1, 100000),
            (USER2, 300000),
        ],
    }
    .assimilate_storage(&mut storage);

    let _ = pallet_eth_sub_bridge::GenesisConfig::<Test> {
        validators_count: 3u32,
        validator_accounts: vec![V1, V2, V3],
        current_limits: vec![100, 200, 50, 400, 10],
    }
    .assimilate_storage(&mut storage);

    let ext = sp_io::TestExternalities::from(storage);
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        Bridge::on_finalize(System::block_number());
        Timestamp::set_timestamp(6 * n);
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}
