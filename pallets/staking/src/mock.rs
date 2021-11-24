// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
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

//! Test utilities

use crate as staking;
use crate::*;
use frame_support::{
    assert_ok, parameter_types,
    traits::{
        Currency, FindAuthor, GenesisBuild, Get, OnFinalize, OnInitialize, OneSessionHandler,
    },
    weights::constants::RocksDbWeight,
    IterableStorageMap, StorageDoubleMap, StorageValue,
};
use node_primitives::Moment;
use pallet_credit::{CreditData, CreditLevel, CreditSetting};
use sp_core::H256;
use sp_io;
use sp_runtime::{
    testing::{Header, TestXt, UintAuthorityId},
    traits::{IdentityLookup, Zero},
};
use sp_staking::offence::{OffenceDetails, OnOffenceHandler};
use std::{cell::RefCell, collections::HashSet};

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 5000;

/// The AccountId alias in this test module.
pub(crate) type AccountId = u64;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

thread_local! {
    static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
}

/// Another session handler struct to test on_disabled.
pub struct OtherSessionHandler;
impl OneSessionHandler<AccountId> for OtherSessionHandler {
    type Key = UintAuthorityId;

    fn on_genesis_session<'a, I: 'a>(_: I)
    where
        I: Iterator<Item = (&'a AccountId, Self::Key)>,
        AccountId: 'a,
    {
    }

    fn on_new_session<'a, I: 'a>(_: bool, validators: I, _: I)
    where
        I: Iterator<Item = (&'a AccountId, Self::Key)>,
        AccountId: 'a,
    {
        SESSION.with(|x| {
            *x.borrow_mut() = (validators.map(|x| x.0.clone()).collect(), HashSet::new())
        });
    }

    fn on_disabled(validator_index: u32) {
        SESSION.with(|d| {
            let mut d = d.borrow_mut();
            let value = d.0[validator_index as usize];
            d.1.insert(value);
        })
    }
}

impl sp_runtime::BoundToRuntimeAppPublic for OtherSessionHandler {
    type Public = UintAuthorityId;
}

pub fn is_disabled(controller: AccountId) -> bool {
    let stash = Staking::ledger(&controller).unwrap().stash;
    SESSION.with(|d| d.borrow().1.contains(&stash))
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Credit: pallet_credit::{Pallet, Call, Storage, Event<T>, Config<T>},
        Staking: staking::{Pallet, Call, Config<T>, Storage, Event<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        DeeperNode: pallet_deeper_node::{Pallet, Call, Storage, Event<T>, Config<T>},
        Micropayment: pallet_micropayment::{Pallet, Call, Storage, Event<T>},
    }
);

/// Author of block is always 11
pub struct Author11;
impl FindAuthor<AccountId> for Author11 {
    fn find_author<'a, I>(_digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
    {
        Some(11)
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(
            frame_support::weights::constants::WEIGHT_PER_SECOND * 2
        );
    pub const MaxLocks: u32 = 1024;
    pub static SessionsPerEra: SessionIndex = 3;
    pub static ExistentialDeposit: Balance = 1;
    pub static SlashDeferDuration: EraIndex = 0;
    pub static Period: BlockNumber = EPOCH_DURATION_IN_BLOCKS;
    pub static Offset: BlockNumber = 0;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = RocksDbWeight;
    type Origin = Origin;
    type Index = AccountIndex;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}
impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

pub struct TestAccountCreator;

impl pallet_micropayment::AccountCreator<u64> for TestAccountCreator {
    fn create_account(_string: &'static str) -> u64 {
        0
    }
}

parameter_types! {
    pub const SecsPerBlock: u32 = 5u32;
    pub const DataPerDPR: u64 = 1024 * 1024 * 1024 * 1024;
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
pub const INITIAL_CREDIT: u64 = 100;
pub const CREDIT_ATTENUATION_STEP: u64 = 1;
pub const BLOCKS_PER_ERA: BlockNumber = 6 * EPOCH_DURATION_IN_BLOCKS;

parameter_types! {
    pub const CreditCapTwoEras: u8 = 5;
    pub const CreditAttenuationStep: u64 = CREDIT_ATTENUATION_STEP;
    pub const MinCreditToDelegate: u64 = 100;
    pub const MicropaymentToCreditFactor: u128 = 1_000_000_000_000_000;
    pub const BlocksPerEra: BlockNumber =  BLOCKS_PER_ERA;
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
}

parameter_types! {
    pub const UncleGenerations: u64 = 0;
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(25);
}
sp_runtime::impl_opaque_keys! {
    pub struct SessionKeys {
        pub other: OtherSessionHandler,
    }
}
impl pallet_session::Config for Test {
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
    type Keys = SessionKeys;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionHandler = (OtherSessionHandler,);
    type Event = Event;
    type ValidatorId = AccountId;
    type ValidatorIdOf = crate::StashOf<Test>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type WeightInfo = ();
}

impl pallet_session::historical::Config for Test {
    type FullIdentification = crate::Exposure<AccountId, Balance>;
    type FullIdentificationOf = crate::ExposureOf<Test>;
}
impl pallet_authorship::Config for Test {
    type FindAuthor = Author11;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = Pallet<Test>;
}
parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const BondingDuration: EraIndex = 3;
}

thread_local! {
    pub static REWARD_REMAINDER_UNBALANCED: RefCell<u128> = RefCell::new(0);
}

pub struct RewardRemainderMock;

impl OnUnbalanced<NegativeImbalanceOf<Test>> for RewardRemainderMock {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
        REWARD_REMAINDER_UNBALANCED.with(|v| {
            *v.borrow_mut() += amount.peek();
        });
        drop(amount);
    }
}

pub struct NumberCurrencyConverter;
impl Convert<u128, Balance> for NumberCurrencyConverter {
    fn convert(x: u128) -> Balance {
        x
    }
}

pub const TOTAL_MINING_REWARD: u128 = 6_000_000_000_000_000_000_000_000;

parameter_types! {
    pub const MiningReward: u128 = TOTAL_MINING_REWARD;
    pub const MaxDelegates: usize = 10;
}

impl Config for Test {
    type BlocksPerEra = BlocksPerEra;
    type Currency = Balances;
    type UnixTime = Timestamp;
    type Event = Event;
    type Slash = ();
    type SessionsPerEra = SessionsPerEra;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type BondingDuration = BondingDuration;
    type SessionInterface = Self;
    type Call = Call;
    type WeightInfo = ();
    type CreditInterface = Credit;
    type NodeInterface = DeeperNode;
    type MaxDelegates = MaxDelegates;
    type NumberToCurrency = NumberCurrencyConverter;
    type TotalMiningReward = MiningReward;
    type ExistentialDeposit = ExistentialDeposit;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

pub type Extrinsic = TestXt<Call, ()>;

pub struct ExtBuilder {
    validator_pool: bool,
    validator_count: u32,
    minimum_validator_count: u32,
    fair: bool,
    num_validators: Option<u32>,
    invulnerables: Vec<AccountId>,
    has_stakers: bool,
    initialize_first_session: bool,
    num_delegators: Option<u32>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            validator_pool: false,
            validator_count: 2,
            minimum_validator_count: 0,
            fair: true,
            num_validators: None,
            invulnerables: vec![],
            has_stakers: true,
            initialize_first_session: true,
            num_delegators: None,
        }
    }
}

impl ExtBuilder {
    pub fn existential_deposit(self, existential_deposit: Balance) -> Self {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = existential_deposit);
        self
    }
    pub fn validator_pool(mut self, validator_pool: bool) -> Self {
        self.validator_pool = validator_pool;
        self
    }
    pub fn validator_count(mut self, count: u32) -> Self {
        self.validator_count = count;
        self
    }
    pub fn minimum_validator_count(mut self, count: u32) -> Self {
        self.minimum_validator_count = count;
        self
    }
    pub fn slash_defer_duration(self, eras: EraIndex) -> Self {
        SLASH_DEFER_DURATION.with(|v| *v.borrow_mut() = eras);
        self
    }
    pub fn fair(mut self, is_fair: bool) -> Self {
        self.fair = is_fair;
        self
    }
    pub fn num_validators(mut self, num_validators: u32) -> Self {
        self.num_validators = Some(num_validators);
        self
    }
    pub fn invulnerables(mut self, invulnerables: Vec<AccountId>) -> Self {
        self.invulnerables = invulnerables;
        self
    }
    pub fn session_per_era(self, length: SessionIndex) -> Self {
        SESSIONS_PER_ERA.with(|v| *v.borrow_mut() = length);
        self
    }
    pub fn period(self, length: BlockNumber) -> Self {
        PERIOD.with(|v| *v.borrow_mut() = length);
        self
    }
    pub fn has_stakers(mut self, has: bool) -> Self {
        self.has_stakers = has;
        self
    }
    pub fn initialize_first_session(mut self, init: bool) -> Self {
        self.initialize_first_session = init;
        self
    }
    pub fn offset(self, offset: BlockNumber) -> Self {
        OFFSET.with(|v| *v.borrow_mut() = offset);
        self
    }
    pub fn num_delegators(mut self, num_delegators: u32) -> Self {
        self.num_delegators = Some(num_delegators);
        self
    }
    pub fn build(self) -> sp_io::TestExternalities {
        sp_tracing::try_init_simple();
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        let balance_factor = if ExistentialDeposit::get() > 1 {
            256
        } else {
            1
        };

        let num_delegators = self.num_delegators.unwrap_or(1);
        let mut user_credit_data = (0..num_delegators)
            .map(|x| {
                (
                    (1001 + x) as AccountId,
                    CreditData {
                        campaign_id: 0,
                        // some tests require delegators to survive one slash
                        credit: INITIAL_CREDIT + CREDIT_ATTENUATION_STEP,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 100,
                    },
                )
            })
            .collect::<Vec<_>>();
        user_credit_data.push((
            1000,
            CreditData {
                campaign_id: 0,
                credit: 99,
                initial_credit_level: CreditLevel::Zero,
                rank_in_initial_credit_level: 0u32,
                number_of_referees: 0,
                current_credit_level: CreditLevel::Zero,
                reward_eras: 1,
            },
        ));
        const MILLICENTS: Balance = 10_000_000_000_000;
        const CENTS: Balance = 1_000 * MILLICENTS;
        const DOLLARS: Balance = 100 * CENTS;
        const DPR: Balance = DOLLARS;
        pallet_credit::GenesisConfig::<Test> {
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
                    tax_rate: Percent::from_percent(10),
                    max_referees_with_rewards: 1,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Two,
                    staking_balance: 46_800 * DPR,
                    base_apy: Percent::from_percent(40),
                    bonus_apy: Percent::from_percent(7),
                    max_rank_with_bonus: 1200u32,
                    tax_rate: Percent::from_percent(10),
                    max_referees_with_rewards: 2,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Three,
                    staking_balance: 76_800 * DPR,
                    base_apy: Percent::from_percent(42),
                    bonus_apy: Percent::from_percent(11),
                    max_rank_with_bonus: 1000u32,
                    tax_rate: Percent::from_percent(9),
                    max_referees_with_rewards: 3,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Four,
                    staking_balance: 138_000 * DPR,
                    base_apy: Percent::from_percent(46),
                    bonus_apy: Percent::from_percent(13),
                    max_rank_with_bonus: 800u32,
                    tax_rate: Percent::from_percent(9),
                    max_referees_with_rewards: 7,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Five,
                    staking_balance: 218_000 * DPR,
                    base_apy: Percent::from_percent(50),
                    bonus_apy: Percent::from_percent(16),
                    max_rank_with_bonus: 600u32,
                    tax_rate: Percent::from_percent(8),
                    max_referees_with_rewards: 12,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Six,
                    staking_balance: 288_000 * DPR,
                    base_apy: Percent::from_percent(54),
                    bonus_apy: Percent::from_percent(20),
                    max_rank_with_bonus: 400u32,
                    tax_rate: Percent::from_percent(7),
                    max_referees_with_rewards: 18,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Seven,
                    staking_balance: 368_000 * DPR,
                    base_apy: Percent::from_percent(57),
                    bonus_apy: Percent::from_percent(25),
                    max_rank_with_bonus: 200u32,
                    tax_rate: Percent::from_percent(6),
                    max_referees_with_rewards: 25,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Eight,
                    staking_balance: 468_000 * DPR,
                    base_apy: Percent::from_percent(60),
                    bonus_apy: Percent::from_percent(30),
                    max_rank_with_bonus: 100u32,
                    tax_rate: Percent::from_percent(5),
                    max_referees_with_rewards: 34,
                    reward_per_referee: 18 * DPR,
                },
            ],
            user_credit_data,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (1, 10 * balance_factor),
                (2, 20 * balance_factor),
                (3, 300 * balance_factor),
                (4, 400 * balance_factor),
                (10, balance_factor),
                (11, balance_factor * 1000),
                (20, balance_factor),
                (21, balance_factor * 2000),
                (30, balance_factor),
                (31, balance_factor * 2000),
                (40, balance_factor),
                (41, balance_factor * 2000),
                (50, balance_factor),
                (51, balance_factor * 2000),
                (60, balance_factor),
                (61, balance_factor * 2000),
                (70, balance_factor),
                (71, balance_factor * 2000),
                (80, balance_factor),
                (81, balance_factor * 2000),
                (100, 2000 * balance_factor),
                (101, 2000 * balance_factor),
                // This allows us to have a total_payout different from 0.
                (999, 1_000_000_000_000),
            ],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let num_validators = self.num_validators.unwrap_or(self.validator_count);
        // Check that the number of validators is sensible.
        assert!(num_validators <= 8);
        let validators = (0..num_validators)
            .map(|x| ((x + 1) * 10 + 1) as AccountId)
            .collect::<Vec<_>>();

        let mut stakers = vec![];
        if self.has_stakers {
            let stake_21 = if self.fair { 1000 } else { 2000 };
            let stake_31 = if self.validator_pool {
                balance_factor * 1000
            } else {
                1
            };
            let status_41 = if self.validator_pool {
                StakerStatus::Validator
            } else {
                StakerStatus::Idle
            };
            stakers = vec![
                // (stash, controller, staked_amount, status)
                (11, 10, balance_factor * 1000, StakerStatus::Validator),
                (21, 20, stake_21, StakerStatus::Validator),
                (31, 30, stake_31, StakerStatus::Validator),
                (41, 40, balance_factor * 1000, status_41),
                (101, 100, balance_factor * 500, StakerStatus::Idle),
            ];
        }

        let mut delegations = vec![];
        if self.has_stakers {
            delegations.push((1001, vec![11, 21]));
        }

        staking::GenesisConfig::<Test> {
            stakers: stakers,
            delegations: delegations,
            validator_count: self.validator_count,
            era_validator_reward: TOTAL_MINING_REWARD / 100_000,
            minimum_validator_count: self.minimum_validator_count,
            invulnerables: self.invulnerables,
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        pallet_session::GenesisConfig::<Test> {
            keys: validators
                .iter()
                .map(|x| {
                    (
                        *x,
                        *x,
                        SessionKeys {
                            other: UintAuthorityId(*x as u64),
                        },
                    )
                })
                .collect(),
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let mut ext = sp_io::TestExternalities::from(storage);
        ext.execute_with(|| {
            let validators = Session::validators();
            SESSION.with(|x| *x.borrow_mut() = (validators.clone(), HashSet::new()));
        });

        if self.initialize_first_session {
            // We consider all test to start after timestamp is initialized This must be ensured by
            // having `timestamp::on_initialize` called before `staking::on_initialize`. Also, if
            // session length is 1, then it is already triggered.
            ext.execute_with(|| {
                System::set_block_number(1);
                Session::on_initialize(1);
                Staking::on_initialize(1);
                Timestamp::set_timestamp(INIT_TIMESTAMP);
            });
        }

        ext.execute_with(pre_conditions);

        ext
    }
    pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
        let mut ext = self.build();
        ext.execute_with(test);
        ext.execute_with(post_conditions);
    }
}

fn pre_conditions() {
    // (BLOCKS_PER_ERA + 1) delegators are enough for all the tests now
    for account in 1000..1001 + BLOCKS_PER_ERA {
        assert_ok!(DeeperNode::im_online(Origin::signed(account as u64)));
    }
}

fn post_conditions() {
    check_exposures();
    check_ledgers();
}

fn check_ledgers() {
    // check the ledger of all stakers.
    Bonded::<Test>::iter().for_each(|(_, ctrl)| assert_ledger_consistent(ctrl))
}

fn check_exposures() {
    // a check per validator to ensure the exposure struct is always sane.
    let era = active_era();
    ErasStakers::<Test>::iter_prefix_values(era).for_each(|expo| {
        assert_eq!(
            expo.total as u128, expo.own as u128,
            "wrong total exposure.",
        );
    })
}

fn assert_ledger_consistent(ctrl: AccountId) {
    // ensures ledger.total == ledger.active + sum(ledger.unlocking).
    let ledger = Staking::ledger(ctrl).expect("Not a controller.");
    let real_total: Balance = ledger
        .unlocking
        .iter()
        .fold(ledger.active, |a, c| a + c.value);
    assert_eq!(real_total, ledger.total);
    assert!(
        ledger.active >= Balances::minimum_balance() || ledger.active == 0,
        "{}: active ledger amount ({}) must be greater than ED {}",
        ctrl,
        ledger.active,
        Balances::minimum_balance()
    );
}

pub(crate) fn active_era() -> EraIndex {
    Staking::active_era().unwrap().index
}

pub(crate) fn current_era() -> EraIndex {
    Staking::current_era().unwrap()
}

pub(crate) fn bond_validator(stash: AccountId, ctrl: AccountId, val: Balance) {
    let _ = Balances::make_free_balance_be(&stash, val);
    let _ = Balances::make_free_balance_be(&ctrl, val);
    assert_ok!(Staking::bond(
        Origin::signed(stash),
        ctrl,
        val,
        RewardDestination::Controller,
    ));
    assert_ok!(Staking::validate(
        Origin::signed(ctrl),
        ValidatorPrefs::default()
    ));
}

/// Progress to the given block, triggering session and era changes as we progress.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
    Staking::on_finalize(System::block_number());
    for b in (System::block_number() + 1)..=n {
        System::set_block_number(b);
        Session::on_initialize(b);
        Staking::on_initialize(b);
        Timestamp::set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);
        if b != n {
            Staking::on_finalize(System::block_number());
        }
    }
}

/// Progresses from the current block number (whatever that may be) to the `P * session_index + 1`.
pub(crate) fn start_session(session_index: SessionIndex) {
    let end: u64 = if Offset::get().is_zero() {
        (session_index as u64) * Period::get()
    } else {
        Offset::get() + (session_index.saturating_sub(1) as u64) * Period::get()
    };
    run_to_block(end);
    // session must have progressed properly.
    assert_eq!(
        Session::current_index(),
        session_index,
        "current session index = {}, expected = {}",
        Session::current_index(),
        session_index,
    );
}

/// Go one session forward.
pub(crate) fn advance_session() {
    let current_index = Session::current_index();
    start_session(current_index + 1);
}

/// Progress until the given era.
pub(crate) fn start_active_era(era_index: EraIndex) {
    start_session((era_index * <SessionsPerEra as Get<u32>>::get()).into());
    assert_eq!(active_era(), era_index);
    // One way or another, current_era must have changed before the active era, so they must match
    // at this point.
    assert_eq!(current_era(), active_era());
}

pub(crate) fn validator_controllers() -> Vec<AccountId> {
    Session::validators()
        .into_iter()
        .map(|s| Staking::bonded(&s).expect("no controller for validator"))
        .collect()
}

pub(crate) fn on_offence_in_era(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_fraction: &[Perbill],
    era: EraIndex,
) {
    let bonded_eras = crate::BondedEras::get();
    for &(bonded_era, start_session) in bonded_eras.iter() {
        if bonded_era == era {
            let _ = Staking::on_offence(offenders, slash_fraction, start_session);
            return;
        } else if bonded_era > era {
            break;
        }
    }

    if Staking::active_era().unwrap().index == era {
        let _ = Staking::on_offence(
            offenders,
            slash_fraction,
            Staking::eras_start_session_index(era).unwrap(),
        );
    } else {
        panic!("cannot slash in era {}", era);
    }
}

pub(crate) fn on_offence_now(
    offenders: &[OffenceDetails<
        AccountId,
        pallet_session::historical::IdentificationTuple<Test>,
    >],
    slash_fraction: &[Perbill],
) {
    let now = Staking::active_era().unwrap().index;
    on_offence_in_era(offenders, slash_fraction, now)
}

pub(crate) fn add_slash(who: &AccountId) {
    on_offence_now(
        &[OffenceDetails {
            offender: (
                who.clone(),
                Staking::eras_stakers(Staking::active_era().unwrap().index, who.clone()),
            ),
            reporters: vec![],
        }],
        &[Perbill::from_percent(10)],
    );
}

pub(crate) fn balances(who: &AccountId) -> (Balance, Balance) {
    (Balances::free_balance(who), Balances::reserved_balance(who))
}

#[macro_export]
macro_rules! assert_session_era {
    ($session:expr, $era:expr) => {
        assert_eq!(
            Session::current_index(),
            $session,
            "wrong session {} != {}",
            Session::current_index(),
            $session,
        );
        assert_eq!(
            Staking::active_era().unwrap().index,
            $era,
            "wrong active era {} != {}",
            Staking::active_era().unwrap().index,
            $era,
        );
    };
}
