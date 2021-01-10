use crate::{Module, Trait};
use frame_support::{
    impl_outer_origin, parameter_types, weights::constants::RocksDbWeight as DbWeight,
    weights::Weight,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Convert, IdentityLookup},
    Perbill,
};

use node_primitives::{Balance, BlockNumber, Moment};

pub type Credit = pallet_credit::Module<Test>;

pub(crate) type AccountId = u64;

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
    type AccountId = AccountId;
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

pub const MILLISECS_PER_BLOCK: Moment = 3000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);

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
    pub const ExistentialDeposit: Balance = 100_000_000_000;
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

impl pallet_micropayment::Trait for Test {
    type Event = ();
    type Currency = Balances;
}
impl pallet_deeper_node::Trait for Test {
    type Event = ();
    type Currency = Balances;
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

parameter_types! {
    pub const BlocksPerEra: BlockNumber = 6 * EPOCH_DURATION_IN_BLOCKS;
}
impl pallet_credit::Trait for Test {
    type Event = ();
    type BlocksPerEra = BlocksPerEra;
    type CurrencyToVote = CurrencyToNumberHandler;
}

impl Trait for Test {
    type Event = ();
    type CreditInterface = Credit;
    type Currency = Balances;
}

pub type Delegating = Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
