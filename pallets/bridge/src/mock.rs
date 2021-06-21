use crate::GenesisConfig;
use crate::{Module, Trait};
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_support::{
    impl_outer_origin, parameter_types, weights::constants::RocksDbWeight as DbWeight,
    weights::Weight,
};
use frame_system as system;
use node_primitives::{Balance, Moment};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

pub type System = frame_system::Module<Test>;

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

pub type Balances = pallet_balances::Module<Test>;

pub struct BalancesWeightInfo;
impl pallet_balances::WeightInfo for BalancesWeightInfo {
    fn transfer() -> Weight {
        (65949000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn transfer_keep_alive() -> Weight {
        (46665000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_balance_creating() -> Weight {
        (27086000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn set_balance_killing() -> Weight {
        (33424000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn force_transfer() -> Weight {
        (65343000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 500;
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type WeightInfo = BalancesWeightInfo;
}

parameter_types! {
    pub const MinimumPeriod: Moment = 2500u64;
}

impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = ();
    type Currency = Balances;
}

pub type BridgeModule = Module<Test>;
type TimestampModule = timestamp::Module<Test>;

const V1: u64 = 1;
const V2: u64 = 2;
const V3: u64 = 3;
const USER1: u64 = 5;
const USER2: u64 = 6;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
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
    let _ = GenesisConfig::<Test> {
        validators_count: 3u32,
        validator_accounts: vec![V1, V2, V3],
        current_limits: vec![100, 200, 50, 400, 1],
    }
    .assimilate_storage(&mut storage);

    let ext = sp_io::TestExternalities::from(storage);
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        BridgeModule::on_finalize(System::block_number());
        TimestampModule::set_timestamp(6 * n);
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}
