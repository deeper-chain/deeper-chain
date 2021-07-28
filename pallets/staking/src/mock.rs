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
    assert_ok,
    pallet_prelude::GenesisBuild,
    parameter_types,
    traits::{Currency, FindAuthor, Get, OnFinalize, OnInitialize, OneSessionHandler},
    weights::{constants::RocksDbWeight, Weight},
    IterableStorageMap, StorageDoubleMap, StorageMap, StorageValue,
};
use node_primitives::Moment;
use pallet_credit::{CreditData, CreditLevel};
use sp_core::H256;
use sp_io;
use sp_npos_elections::{
    reduce, to_support_map, ElectionScore, EvaluateSupport, ExtendedBalance, StakedAssignment,
};
use sp_runtime::{
    curve::PiecewiseLinear,
    testing::{Header, TestXt, UintAuthorityId},
    traits::{IdentityLookup, Zero},
};
use sp_staking::offence::{OffenceDetails, OnOffenceHandler};
use std::{cell::RefCell, collections::HashSet};

pub const INIT_TIMESTAMP: u64 = 30_000;
pub const BLOCK_TIME: u64 = 1000;

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

    fn on_disabled(validator_index: usize) {
        SESSION.with(|d| {
            let mut d = d.borrow_mut();
            let value = d.0[validator_index];
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
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Credit: pallet_credit::{Module, Call, Storage, Event<T>, Config<T>},
        Staking: staking::{Module, Call, Config<T>, Storage, Event<T>},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        DeeperNode: pallet_deeper_node::{Module, Call, Storage, Event<T>, Config<T> },
        Micropayment: pallet_micropayment::{Module, Call, Storage, Event<T>},
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
    pub static ElectionLookahead: BlockNumber = 0;
    pub static Period: BlockNumber = 5;
    pub static Offset: BlockNumber = 0;
    pub static MaxIterations: u32 = 0;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
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

parameter_types! {
    pub const CreditInitScore: u64 = 60;
    pub const MaxCreditScore: u64 = 800;
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
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
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
    type EventHandler = Module<Test>;
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
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub const UnsignedPriority: u64 = 1 << 20;
    pub const MinSolutionScoreBump: Perbill = Perbill::zero();
    pub OffchainSolutionWeightLimit: Weight = BlockWeights::get().max_block;
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

const GENESIS_BLOCK_REWARD: u128 = 90_000_000_000_000_000;
const TOTAL_MINING_REWARD: u128 = 6_000_000_000_000_000_000_000_000;

parameter_types! {
    pub const RewardAdjustPeriod: u32 = 4;
    pub const CreditToTokenFactor: u128 = 500_000_000_000_000;
    pub const RewardAdjustFactor: u128 = 77_760_000;
    pub const RewardPerBlock: u128 = GENESIS_BLOCK_REWARD / 30;
    pub const MiningReward: u128 = TOTAL_MINING_REWARD;
    pub const MaxValidatorsCanSelected: usize = 10;
}

impl Config for Test {
    type Currency = Balances;
    type UnixTime = Timestamp;
    type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
    type RewardRemainder = RewardRemainderMock;
    type Event = Event;
    type Slash = ();
    type Reward = ();
    type SessionsPerEra = SessionsPerEra;
    type SlashDeferDuration = SlashDeferDuration;
    type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
    type BondingDuration = BondingDuration;
    type SessionInterface = Self;
    type NextNewSession = Session;
    type ElectionLookahead = ElectionLookahead;
    type Call = Call;
    type MaxIterations = MaxIterations;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type UnsignedPriority = UnsignedPriority;
    type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
    type WeightInfo = ();
    type CreditInterface = Credit;
    type NodeInterface = DeeperNode;
    type MaxValidatorsCanSelected = MaxValidatorsCanSelected;
    type CurrencyToNumber = CurrencyToNumberHandler;
    type CreditToTokenFactor = CreditToTokenFactor;
    type RewardAdjustFactor = RewardAdjustFactor;
    type RewardPerBlock = RewardPerBlock;
    type RewardAdjustPeriod = RewardAdjustPeriod;
    type BlocksPerEra = BlocksPerEra;
    type RemainderMiningReward = MiningReward;
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
    nominate: bool,
    validator_count: u32,
    minimum_validator_count: u32,
    fair: bool,
    num_validators: Option<u32>,
    invulnerables: Vec<AccountId>,
    has_stakers: bool,
    initialize_first_session: bool,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            validator_pool: false,
            nominate: true,
            validator_count: 2,
            minimum_validator_count: 0,
            fair: true,
            num_validators: None,
            invulnerables: vec![],
            has_stakers: true,
            initialize_first_session: true,
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
    pub fn nominate(mut self, nominate: bool) -> Self {
        self.nominate = nominate;
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
    pub fn election_lookahead(self, look: BlockNumber) -> Self {
        ELECTION_LOOKAHEAD.with(|v| *v.borrow_mut() = look);
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
    pub fn max_offchain_iterations(self, iterations: u32) -> Self {
        MAX_ITERATIONS.with(|v| *v.borrow_mut() = iterations);
        self
    }
    pub fn offchain_election_ext(self) -> Self {
        self.session_per_era(4).period(5).election_lookahead(3)
    }
    pub fn initialize_first_session(mut self, init: bool) -> Self {
        self.initialize_first_session = init;
        self
    }
    pub fn offset(self, offset: BlockNumber) -> Self {
        OFFSET.with(|v| *v.borrow_mut() = offset);
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

        let num_validators = self.num_validators.unwrap_or(self.validator_count);
        // Check that the number of validators is sensible.
        assert!(num_validators <= 8);
        let validators = (0..num_validators)
            .map(|x| ((x + 1) * 10 + 1) as AccountId)
            .collect::<Vec<_>>();

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

        pub const BLOCKS_PER_ERA: u64 = 178000 as u64;
        pallet_credit::GenesisConfig::<Test> {
            credit_settings: vec![],
            user_credit_data: vec![
                (
                    1,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    2,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    3,
                    CreditData {
                        credit: 105,
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
                    10,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    11,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    19,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    20,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    22,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    30,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    40,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
                (
                    100,
                    CreditData {
                        credit: 105,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        expiration: BLOCKS_PER_ERA,
                    },
                ),
            ],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let mut stakers = vec![];
        if self.has_stakers {
            let stake_21 = if self.fair { 1000 } else { 2000 };
            let stake_31 = if self.validator_pool {
                balance_factor * 1000
            } else {
                1
            };
            let status_41 = if self.validator_pool {
                StakerStatus::<AccountId>::Validator
            } else {
                StakerStatus::<AccountId>::Idle
            };
            let nominated = if self.nominate { vec![11, 21] } else { vec![] };
            stakers = vec![
                // (stash, controller, staked_amount, status)
                (
                    11,
                    10,
                    balance_factor * 1000,
                    StakerStatus::<AccountId>::Validator,
                ),
                (21, 20, stake_21, StakerStatus::<AccountId>::Validator),
                (31, 30, stake_31, StakerStatus::<AccountId>::Validator),
                (41, 40, balance_factor * 1000, status_41),
                (
                    101,
                    100,
                    balance_factor * 500,
                    StakerStatus::<AccountId>::Idle,
                ),
            ];
        }

        staking::GenesisConfig::<Test> {
            stakers: stakers,
            validator_count: self.validator_count,
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

        ext
    }
    pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
        let mut ext = self.build();
        ext.execute_with(test);
        ext.execute_with(post_conditions);
    }
}

fn post_conditions() {
    //check_nominators();
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
            expo.total as u128,
            expo.own as u128 + expo.others.iter().map(|e| e.value as u128).sum::<u128>(),
            "wrong total exposure.",
        );
    })
}

fn assert_is_stash(acc: AccountId) {
    assert!(Staking::bonded(&acc).is_some(), "Not a stash.");
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

/// Time it takes to finish a session.
///
/// Note, if you see `time_per_session() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_session() -> u64 {
    Period::get() * BLOCK_TIME
}

/// Time it takes to finish an era.
///
/// Note, if you see `time_per_era() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_era() -> u64 {
    time_per_session() * SessionsPerEra::get() as u64
}

/// Time that will be calculated for the reward per era.
pub(crate) fn reward_time_per_era() -> u64 {
    time_per_era() - BLOCK_TIME
}

pub(crate) fn reward_all_elected() {
    let rewards = <Test as Config>::SessionInterface::validators()
        .into_iter()
        .map(|v| (v, 1));

    <Module<Test>>::reward_by_ids(rewards)
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
            let _ = Staking::on_offence(offenders, slash_fraction, start_session).unwrap();
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
        )
        .unwrap();
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

pub(crate) fn staking_events() -> Vec<staking::Event<Test>> {
    System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::staking(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn balances(who: &AccountId) -> (Balance, Balance) {
    (Balances::free_balance(who), Balances::reserved_balance(who))
}
