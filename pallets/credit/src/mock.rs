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
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Event<T>, Config<T>},
        Credit: pallet_credit::{Pallet, Call, Storage, Event<T>},
        DeeperNode: pallet_deeper_node::{Pallet, Call, Storage, Event<T>, Config<T> },
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything; // modified by james.soong
    type OnSetCode = ();
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

pub const MILLISECS_PER_BLOCK: Moment = 5000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
pub const BLOCKS_PER_ERA: u64 = (6 * EPOCH_DURATION_IN_BLOCKS) as u64;
pub const CREDIT_ATTENUATION_STEP: u64 = 1;
pub const CREDIT_CAP_TWO_ERAS: u8 = 1;

parameter_types! {
    pub const CreditCapTwoEras: u8 = CREDIT_CAP_TWO_ERAS;
    pub const CreditAttenuationStep: u64 = CREDIT_ATTENUATION_STEP;
    pub const MinCreditToDelegate: u64 = 100;
    pub const MicropaymentToCreditFactor: u128 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  6 * EPOCH_DURATION_IN_BLOCKS;
}

impl pallet_credit::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type BlocksPerEra = BlocksPerEra;
    type CreditCapTwoEras = CreditCapTwoEras;
    type CreditAttenuationStep = CreditAttenuationStep;
    type MinCreditToDelegate = MinCreditToDelegate;
    type MicropaymentToCreditFactor = MicropaymentToCreditFactor;
    type NodeInterface = DeeperNode;
    type WeightInfo = ();
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
                campaign_id: 0,
                credit_level: CreditLevel::Zero,
                staking_balance: 0,
                base_apy: Percent::from_percent(0),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 0,
                reward_per_referee: 0,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::One,
                staking_balance: 20_000 * DPR,
                base_apy: Percent::from_percent(39),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 1,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Two,
                staking_balance: 46_800 * DPR,
                base_apy: Percent::from_percent(47),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 2,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Three,
                staking_balance: 76_800 * DPR,
                base_apy: Percent::from_percent(53),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 3,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Four,
                staking_balance: 138_000 * DPR,
                base_apy: Percent::from_percent(59),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 7,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Five,
                staking_balance: 218_000 * DPR,
                base_apy: Percent::from_percent(66),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 12,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Six,
                staking_balance: 288_000 * DPR,
                base_apy: Percent::from_percent(74),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 18,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Seven,
                staking_balance: 368_000 * DPR,
                base_apy: Percent::from_percent(82),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 25,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 0,
                credit_level: CreditLevel::Eight,
                staking_balance: 468_000 * DPR,
                base_apy: Percent::from_percent(90),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 34,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Zero,
                staking_balance: 0,
                base_apy: Percent::from_percent(0),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 0,
                reward_per_referee: 0,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::One,
                staking_balance: 20_000 * DPR,
                base_apy: Percent::from_percent(39),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 1,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Two,
                staking_balance: 46_800 * DPR,
                base_apy: Percent::from_percent(44),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 2,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Three,
                staking_balance: 76_800 * DPR,
                base_apy: Percent::from_percent(50),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 3,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Four,
                staking_balance: 138_000 * DPR,
                base_apy: Percent::from_percent(56),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 7,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Five,
                staking_balance: 218_000 * DPR,
                base_apy: Percent::from_percent(62),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 12,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Six,
                staking_balance: 288_000 * DPR,
                base_apy: Percent::from_percent(69),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 18,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Seven,
                staking_balance: 368_000 * DPR,
                base_apy: Percent::from_percent(75),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 25,
                reward_per_referee: 0 * DPR,
            },
            CreditSetting {
                campaign_id: 1,
                credit_level: CreditLevel::Eight,
                staking_balance: 468_000 * DPR,
                base_apy: Percent::from_percent(80),
                bonus_apy: Percent::from_percent(0),
                max_rank_with_bonus: 0u32,
                tax_rate: Percent::from_percent(0),
                max_referees_with_rewards: 34,
                reward_per_referee: 0 * DPR,
            },
        ],
        user_credit_data: vec![
            (
                1,
                CreditData {
                    campaign_id: 0,
                    credit: 0,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::Zero,
                    reward_eras: 1,
                },
            ),
            (
                2,
                CreditData {
                    campaign_id: 0,
                    credit: 0,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::Zero,
                    reward_eras: 1,
                },
            ),
            (
                3,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 1,
                },
            ),
            (
                4,
                CreditData {
                    campaign_id: 0,
                    credit: 0,
                    initial_credit_level: CreditLevel::Zero,
                    rank_in_initial_credit_level: 0u32,
                    number_of_referees: 0,
                    current_credit_level: CreditLevel::Zero,
                    reward_eras: 0,
                },
            ),
            (
                5,
                CreditData {
                    campaign_id: 0,
                    credit: 0,
                    initial_credit_level: CreditLevel::Zero,
                    rank_in_initial_credit_level: 0u32,
                    number_of_referees: 0,
                    current_credit_level: CreditLevel::Zero,
                    reward_eras: 0,
                },
            ),
            (
                6,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 270,
                },
            ),
            (
                7,
                CreditData {
                    campaign_id: 0,
                    credit: 400,
                    initial_credit_level: CreditLevel::Four,
                    rank_in_initial_credit_level: 80u32,
                    number_of_referees: 7,
                    current_credit_level: CreditLevel::Four,
                    reward_eras: 270,
                },
            ),
            (
                8,
                CreditData {
                    campaign_id: 0,
                    credit: 400,
                    initial_credit_level: CreditLevel::Four,
                    rank_in_initial_credit_level: 801u32,
                    number_of_referees: 7,
                    current_credit_level: CreditLevel::Four,
                    reward_eras: 270,
                },
            ),
            (
                9,
                CreditData {
                    campaign_id: 0,
                    credit: 400,
                    initial_credit_level: CreditLevel::Four,
                    rank_in_initial_credit_level: 800u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::Four,
                    reward_eras: 270,
                },
            ),
            (
                10,
                CreditData {
                    campaign_id: 0,
                    credit: 400,
                    initial_credit_level: CreditLevel::Four,
                    rank_in_initial_credit_level: 801u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::Four,
                    reward_eras: 270,
                },
            ),
            (
                11,
                CreditData {
                    campaign_id: 1,
                    credit: 200,
                    initial_credit_level: CreditLevel::Two,
                    rank_in_initial_credit_level: 801u32,
                    number_of_referees: 2,
                    current_credit_level: CreditLevel::Two,
                    reward_eras: 270,
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
