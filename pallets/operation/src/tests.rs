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

#![cfg(test)]

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, BadOrigin},
    Perbill,
};

use frame_support::traits::{ConstU128, ConstU32};
use frame_support::{
    assert_noop, assert_ok, pallet_prelude::GenesisBuild, parameter_types, weights::Weight,
};
use node_primitives::{Balance, Moment};
use pallet_credit::{CreditData, CreditLevel, CreditSetting};
use sp_runtime::Percent;

use super::*;
use crate::{self as pallet_operation};
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
        Operation: pallet_operation::{Pallet, Call, Event<T>},
        Credit: pallet_credit::{Pallet, Call, Storage, Event<T>, Config<T>},
        Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        DeeperNode: pallet_deeper_node::{Pallet, Call, Storage, Event<T>, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
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
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u128;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxMember: u32 = 100;
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

type BlockNumber = u64;
type AccountId = u128;

pub const MILLISECS_PER_BLOCK: Moment = 5000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
pub const CREDIT_ATTENUATION_STEP: u64 = 1;
pub const CREDIT_CAP_TWO_ERAS: u8 = 1;

parameter_types! {
    pub const CreditCapTwoEras: u8 = CREDIT_CAP_TWO_ERAS;
    pub const CreditAttenuationStep: u64 = CREDIT_ATTENUATION_STEP;
    pub const MinCreditToDelegate: u64 = 100;
    pub const MicropaymentToCreditFactor: u128 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  6 * EPOCH_DURATION_IN_BLOCKS;
    pub const SecsPerBlock: u32 = 5u32;
    pub const DPRPerCreditBurned: u64 = 50;
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

parameter_types! {
    pub const MinimumPeriod: Moment = 5u64;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
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
    type UnixTime = Timestamp;
    type SecsPerBlock = SecsPerBlock;
    type DPRPerCreditBurned = DPRPerCreditBurned;
    type BurnedTo = ();
}

impl Config for Test {
    type Event = Event;
    type WeightInfo = ();
    type MaxMember = MaxMember;
    type Currency = Balances;
    type CreditInterface = Credit;
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
            (
                12,
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
    GenesisBuild::<Test>::assimilate_storage(&genesis_config, &mut t).unwrap();
    t.into()
}

#[test]
fn set_lock_members_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::set_balance(Origin::root(), 1, 1_000, 0));
        assert_ok!(Operation::set_reserve_members(Origin::root(), vec!(2)));
        assert_ok!(Operation::force_reserve_by_member(Some(2).into(), 1, 500));
        assert_eq!(Balances::free_balance(&1), 500);
        assert_ok!(Balances::force_unreserve(Origin::root(), 1, 500));
        assert_eq!(Balances::free_balance(&1), 1000);
    });
}

#[test]
fn update_nft_class_credit() {
    new_test_ext().execute_with(|| {
        assert_noop!(Operation::update_nft_class_credit(Origin::signed(1), 0, 5), BadOrigin);

        assert_ok!(Operation::update_nft_class_credit(Origin::root(), 0, 5));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(0), 5);

        assert_ok!(Operation::update_nft_class_credit(Origin::root(), 1, 10));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(1), 10);
    });
}

#[test]
fn burn_nft() {
    new_test_ext().execute_with(|| {
        assert_ok!(pallet_uniques::Pallet::<Test>::force_create(
            Origin::root(),
            0,
            1,
            true
        ));
        assert_ok!(pallet_uniques::Pallet::<Test>::force_create(
            Origin::root(),
            1,
            1,
            true
        ));
        assert_ok!(pallet_uniques::Pallet::<Test>::force_create(
            Origin::root(),
            2,
            1,
            true
        ));

        assert_ok!(Uniques::mint(Origin::signed(1), 0, 42, 1));
        assert_ok!(Uniques::mint(Origin::signed(1), 1, 42, 1));
        assert_ok!(Uniques::mint(Origin::signed(1), 2, 42, 1));

        assert_noop!(
            Operation::brun_nft(Origin::signed(1), 0, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );
        assert_noop!(
            Operation::brun_nft(Origin::signed(1), 1, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );
        assert_noop!(
            Operation::brun_nft(Origin::signed(1), 2, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );

        assert_ok!(Operation::update_nft_class_credit(Origin::root(), 0, 5));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(0), 5);
        assert_ok!(Operation::update_nft_class_credit(Origin::root(), 1, 10));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(1), 10);

        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        // update_credit_data works
        assert_ok!(Credit::add_or_update_credit_data(
            Origin::root(),
            1,
            credit_data.clone()
        ));
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 100);

        assert_ok!(Operation::brun_nft(Origin::signed(1), 0, 42));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 105);

        assert_ok!(Operation::brun_nft(Origin::signed(1), 1, 42));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 115);

        assert_noop!(
            Operation::brun_nft(Origin::signed(1), 2, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );
    });
}
