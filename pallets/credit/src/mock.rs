use super::*;
use crate as pallet_credit;
use frame_support::{
    pallet_prelude::GenesisBuild,
    parameter_types,
    traits::{OnFinalize, OnInitialize},
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
    pub const SecsPerBlock: u32 = 5u32;
    pub const DataPerDPR: u64 = 1024 * 1024 * 1024 * 1024;
}
impl pallet_micropayment::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type SecsPerBlock = SecsPerBlock;
    type DataPerDPR = DataPerDPR;
}

parameter_types! {
    pub const MinLockAmt: u32 = 100;
    pub const MaxDurationDays: u8 = 7;
    pub const MaxIpLength: usize = 256;
    pub const DayToBlocknum: u32 = (24 * 3600 * 1000 / 5000) as u32;
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
pub const BLOCKS_PER_ERA: u64 = (6 * EPOCH_DURATION_IN_BLOCKS) as u64;
pub const CREDIT_ATTENUATION_STEP: u64 = 1;
pub const BLOCKS_PER_DAY: u64 = 24 * 60 * 60 / SECS_PER_BLOCK;

parameter_types! {
    pub const InitialCredit: u64 = 100;
    pub const CreditCapPerEra: u8 = 5;
    pub const CreditAttenuationStep: u64 = CREDIT_ATTENUATION_STEP;
    pub const MinCreditToDelegate: u64 = 100;
    pub const MicropaymentToCreditFactor: u64 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  6 * EPOCH_DURATION_IN_BLOCKS;
}

impl pallet_credit::Config for Test {
    type Event = Event;
    type BlocksPerEra = BlocksPerEra;
    type InitialCredit = InitialCredit;
    type CreditCapPerEra = CreditCapPerEra;
    type CreditAttenuationStep = CreditAttenuationStep;
    type MinCreditToDelegate = MinCreditToDelegate;
    type MicropaymentToCreditFactor = MicropaymentToCreditFactor;
    type NodeInterface = DeeperNode;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 500), (2, 500), (3, 500), (4, 500), (5, 500)],
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    const DPR: u128 = 1_000_000_000_000_000_000u128;
    let genesis_config = pallet_credit::GenesisConfig::<Test> {
        credit_settings: vec![
            CreditSetting {
                credit_level: CreditLevel::Zero,
                balance: 0,
                base_apy: Percent::from_percent(0),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 0,
                reward_per_referee: 0,
            },
            CreditSetting {
                credit_level: CreditLevel::One,
                balance: 20_000 * DPR,
                base_apy: Percent::from_percent(39),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(10),
                max_referees_with_rewards: 1,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Two,
                balance: 46_800 * DPR,
                base_apy: Percent::from_percent(40),
                bonus_apy: Percent::from_percent(7),
                max_rank_with_bonus: 1200u32,
                tax_rate: Percent::from_percent(10),
                max_referees_with_rewards: 2,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Three,
                balance: 76_800 * DPR,
                base_apy: Percent::from_percent(42),
                bonus_apy: Percent::from_percent(11),
                max_rank_with_bonus: 1000u32,
                tax_rate: Percent::from_percent(9),
                max_referees_with_rewards: 3,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Four,
                balance: 138_000 * DPR,
                base_apy: Percent::from_percent(46),
                bonus_apy: Percent::from_percent(13),
                max_rank_with_bonus: 800u32,
                tax_rate: Percent::from_percent(9),
                max_referees_with_rewards: 7,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Five,
                balance: 218_000 * DPR,
                base_apy: Percent::from_percent(50),
                bonus_apy: Percent::from_percent(16),
                max_rank_with_bonus: 600u32,
                tax_rate: Percent::from_percent(8),
                max_referees_with_rewards: 12,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Six,
                balance: 288_000 * DPR,
                base_apy: Percent::from_percent(54),
                bonus_apy: Percent::from_percent(20),
                max_rank_with_bonus: 400u32,
                tax_rate: Percent::from_percent(7),
                max_referees_with_rewards: 18,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Seven,
                balance: 368_000 * DPR,
                base_apy: Percent::from_percent(57),
                bonus_apy: Percent::from_percent(25),
                max_rank_with_bonus: 200u32,
                tax_rate: Percent::from_percent(6),
                max_referees_with_rewards: 25,
                reward_per_referee: 18 * DPR,
            },
            CreditSetting {
                credit_level: CreditLevel::Eight,
                balance: 468_000 * DPR,
                base_apy: Percent::from_percent(60),
                bonus_apy: Percent::from_percent(30),
                max_rank_with_bonus: 100u32,
                tax_rate: Percent::from_percent(5),
                max_referees_with_rewards: 34,
                reward_per_referee: 18 * DPR,
            },
        ],
        user_credit_data: vec![
            (
                1,
                CreditData {
                    credit: 0,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    expiration: BLOCKS_PER_ERA,
                },
            ),
            (
                2,
                CreditData {
                    credit: 0,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    expiration: BLOCKS_PER_ERA,
                },
            ),
            (
                3,
                CreditData {
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    expiration: BLOCKS_PER_ERA,
                },
            ),
            (
                4,
                CreditData {
                    credit: 0,
                    initial_credit_level: CreditLevel::Zero,
                    rank_in_initial_credit_level: 0u32,
                    number_of_referees: 0,
                    expiration: 0,
                },
            ),
            (
                5,
                CreditData {
                    credit: 0,
                    initial_credit_level: CreditLevel::Zero,
                    rank_in_initial_credit_level: 0u32,
                    number_of_referees: 0,
                    expiration: 0,
                },
            ),
            (
                6,
                CreditData {
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    expiration: BLOCKS_PER_ERA * 270,
                },
            ),
            (
                7,
                CreditData {
                    credit: 400,
                    initial_credit_level: CreditLevel::Four,
                    rank_in_initial_credit_level: 80u32,
                    number_of_referees: 7,
                    expiration: BLOCKS_PER_ERA * 270,
                },
            ),
        ],
    };
    GenesisBuild::<Test>::assimilate_storage(&genesis_config, &mut storage).unwrap();

    storage.into()
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}
