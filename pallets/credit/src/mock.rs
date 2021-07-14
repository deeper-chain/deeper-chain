use crate as pallet_credit;
use frame_support::{
    parameter_types,
    traits::{OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Convert, IdentityLookup},
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
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Event<T>, Config<T>},
        Micropayment: pallet_micropayment::{Module, Call, Storage, Event<T>},
        Credit: pallet_credit::{Module, Call, Storage, Event<T>},
        DeeperNode: pallet_deeper_node::{Module, Call, Storage, Event<T>, Config<T> },
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
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

parameter_types! {
    pub const DayToBlocknum: u32 = (24 * 3600 * 1000 / 5000) as u32;
    pub const DataPerDPR: u64 = 1024 * 1024 * 1024 * 1024;
}
impl pallet_micropayment::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type DayToBlocknum = DayToBlocknum;
    type DataPerDPR = DataPerDPR;
}

parameter_types! {
    pub const MinLockAmt: u32 = 100;
    pub const MaxDurationDays: u8 = 7;
    pub const MaxIpLength: usize = 256;
}
impl pallet_deeper_node::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type MinLockAmt = MinLockAmt;
    type MaxDurationDays = MaxDurationDays;
    type DayToBlocknum = DayToBlocknum;
    type MaxIpLength = MaxIpLength;
}

pub const MILLISECS_PER_BLOCK: Moment = 5000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
parameter_types! {
    pub const CreditInitScore: u64 = 0;
    pub const MaxCreditScore: u64 = u64::MAX;
    pub const CreditScoreCapPerEra: u8 = 5;
    pub const CreditScoreAttenuationLowerBound: u64 = 40;
    pub const CreditScoreAttenuationStep: u64 = 5;
    pub const CreditScoreDelegatedPermitThreshold: u64 = 100;
    pub const MicropaymentToCreditScoreFactor: u64 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  6 * EPOCH_DURATION_IN_BLOCKS;
}

pub struct CurrencyToNumberHandler;
impl Convert<Balance, u64> for CurrencyToNumberHandler {
    fn convert(x: Balance) -> u64 {
        x as u64
    }
}
impl Convert<u128, Balance> for CurrencyToNumberHandler {
    fn convert(x: u128) -> Balance {
        x
    }
}

impl pallet_credit::Config for Test {
    type Event = Event;
    type BlocksPerEra = BlocksPerEra;
    type CurrencyToVote = CurrencyToNumberHandler;
    type CreditInitScore = CreditInitScore;
    type MaxCreditScore = MaxCreditScore;
    type CreditScoreCapPerEra = CreditScoreCapPerEra;
    type CreditScoreAttenuationLowerBound = CreditScoreAttenuationLowerBound;
    type CreditScoreAttenuationStep = CreditScoreAttenuationStep;
    type CreditScoreDelegatedPermitThreshold = CreditScoreDelegatedPermitThreshold;
    type MicropaymentToCreditScoreFactor = MicropaymentToCreditScoreFactor;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 500), (2, 500), (3, 500), (4, 500), (5, 500)],
    }
    .assimilate_storage(&mut storage);

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
