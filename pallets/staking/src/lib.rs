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

// # Staking Pallet
//
// The Staking module is used to manage funds at stake by network maintainers.

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
pub mod slashing;
#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod testing_utils;
pub mod weights;

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    dispatch::{DispatchResult, DispatchResultWithPostInfo},
    ensure,
    pallet_prelude::*,
    traits::{
        Currency, EnsureOrigin, ExistenceRequirement, Get, Imbalance, IsSubType, LockIdentifier,
        LockableCurrency, OnUnbalanced, UnixTime, WithdrawReasons,
    },
    weights::{
        constants::{WEIGHT_PER_MICROS, WEIGHT_PER_NANOS},
        Weight,
    },
    PalletId,
};
use frame_system::{ensure_root, ensure_signed, offchain::SendTransactionTypes, pallet_prelude::*};
pub use pallet::*;
use pallet_credit::CreditInterface;
use pallet_deeper_node::NodeInterface;
use pallet_operation::OperationInterface;
use pallet_session::historical;
use sp_runtime::{
    traits::{
        AccountIdConversion, AtLeast32BitUnsigned, CheckedSub, Convert, Dispatchable,
        SaturatedConversion, Saturating, StaticLookup, UniqueSaturatedInto, Zero,
    },
    Perbill, Percent, RuntimeDebug,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_staking::{
    offence::{
        DisableStrategy, Offence, OffenceDetails, OffenceError, OnOffenceHandler, ReportOffence,
    },
    SessionIndex,
};
use sp_std::{
    cmp, cmp::Ordering, collections::btree_map::BTreeMap, collections::btree_set::BTreeSet,
    convert::From, convert::TryInto, prelude::*,
};
pub use weights::WeightInfo;

use scale_info::TypeInfo;

const STAKING_ID: LockIdentifier = *b"staking ";
pub const MAX_UNLOCKING_CHUNKS: usize = 32;

pub(crate) const LOG_TARGET: &'static str = "staking";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Counter for the number of "reward" points earned by a given validator.
pub type RewardPoint = u32;

/// The balance type of this module.
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

/// Information regarding the active era (era in used in session).
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ActiveEraInfo {
    /// Index of era.
    pub index: EraIndex,
    /// Moment of start expressed as millisecond from `$UNIX_EPOCH`.
    ///
    /// Start can be none if start hasn't been set for the era yet,
    /// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
    start: Option<u64>,
}

/// Reward points of an era. Used to split era total payout between validators.
///
/// This points will be used to reward validators.
#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct EraRewardPoints<AccountId: Ord> {
    /// Total number of points. Equals the sum of reward points for each validator.
    total: RewardPoint,
    /// The reward points earned by a given validator.
    individual: BTreeMap<AccountId, RewardPoint>,
}

impl<AccountId: Ord> Default for EraRewardPoints<AccountId> {
    fn default() -> Self {
        EraRewardPoints {
            total: Default::default(),
            individual: BTreeMap::new(),
        }
    }
}

/// Indicates the initial status of the staker.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Clone))]
pub enum StakerStatus {
    /// Chilling.
    Idle,
    /// Declared desire in validating or already participating in it.
    Validator,
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum RewardDestination<AccountId> {
    /// Pay into the stash account, increasing the amount at stake accordingly.
    Staked,
    /// Pay into the stash account, not increasing the amount at stake.
    Stash,
    /// Pay into the controller account.
    Controller,
    /// Pay into a specified account.
    Account(AccountId),
}

impl<AccountId> Default for RewardDestination<AccountId> {
    fn default() -> Self {
        RewardDestination::Staked
    }
}

#[derive(Encode, Decode, Default, RuntimeDebug, TypeInfo, Clone, PartialEq)]
pub struct RewardData<Balance: HasCompact> {
    pub total_referee_reward: Balance,
    pub received_referee_reward: Balance,
    pub referee_reward: Balance,
    pub received_pocr_reward: Balance,
    pub poc_reward: Balance,
}

/// Preference of what happens regarding validation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ValidatorPrefs {
    /// not used. It may be removed in future.
    #[codec(compact)]
    pub commission: Perbill,
    /// Whether or not this validator is accepting more delegations. If `true`, then no delegator
    /// who is not already delegating this validator may delegate them. By default, validators
    /// are accepting delegations.
    pub blocked: bool,
}

impl Default for ValidatorPrefs {
    fn default() -> Self {
        ValidatorPrefs {
            commission: Default::default(),
            blocked: false,
        }
    }
}

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct UnlockChunk<Balance: HasCompact> {
    /// Amount of funds to be unlocked.
    #[codec(compact)]
    value: Balance,
    /// Era number at which point it'll be unlocked.
    #[codec(compact)]
    era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct StakingLedger<AccountId, Balance: HasCompact> {
    /// The stash account whose balance is actually locked and at stake.
    pub stash: AccountId,
    /// The total amount of the stash's balance that we are currently accounting for.
    /// It's just `active` plus all the `unlocking` balances.
    #[codec(compact)]
    pub total: Balance,
    /// The total amount of the stash's balance that will be at stake in any forthcoming
    /// rounds.
    #[codec(compact)]
    pub active: Balance,
    /// Any balance that is becoming free, which may eventually be transferred out
    /// of the stash (assuming it doesn't get slashed first).
    pub unlocking: Vec<UnlockChunk<Balance>>,
    /// List of eras for which the stakers behind a validator have claimed rewards. Only updated
    /// for validators.
    pub claimed_rewards: Vec<EraIndex>,
}

impl<AccountId, Balance: HasCompact + Copy + Saturating + AtLeast32BitUnsigned>
    StakingLedger<AccountId, Balance>
{
    /// Remove entries from `unlocking` that are sufficiently old and reduce the
    /// total by the sum of their balances.
    fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
        let mut total = self.total;
        let unlocking = self
            .unlocking
            .into_iter()
            .filter(|chunk| {
                if chunk.era > current_era {
                    true
                } else {
                    total = total.saturating_sub(chunk.value);
                    false
                }
            })
            .collect();

        Self {
            stash: self.stash,
            total,
            active: self.active,
            unlocking,
            claimed_rewards: self.claimed_rewards,
        }
    }

    /// Re-bond funds that were scheduled for unlocking.
    fn rebond(mut self, value: Balance) -> Self {
        let mut unlocking_balance: Balance = Zero::zero();

        while let Some(last) = self.unlocking.last_mut() {
            if unlocking_balance + last.value <= value {
                unlocking_balance += last.value;
                self.active += last.value;
                self.unlocking.pop();
            } else {
                let diff = value - unlocking_balance;

                unlocking_balance += diff;
                self.active += diff;
                last.value -= diff;
            }

            if unlocking_balance >= value {
                break;
            }
        }

        self
    }
}

impl<AccountId, Balance> StakingLedger<AccountId, Balance>
where
    Balance: AtLeast32BitUnsigned + Saturating + Copy,
{
    /// Slash the validator for a given amount of balance. This can grow the value
    /// of the slash in the case that the validator has less than `minimum_balance`
    /// active funds. Returns the amount of funds actually slashed.
    ///
    /// Slashes from `active` funds first, and then `unlocking`, starting with the
    /// chunks that are closest to unlocking.
    fn slash(&mut self, mut value: Balance, minimum_balance: Balance) -> Balance {
        let pre_total = self.total;
        let total = &mut self.total;
        let active = &mut self.active;

        let slash_out_of =
            |total_remaining: &mut Balance, target: &mut Balance, value: &mut Balance| {
                let mut slash_from_target = (*value).min(*target);

                if !slash_from_target.is_zero() {
                    *target -= slash_from_target;

                    // don't leave a dust balance in the staking system.
                    if *target <= minimum_balance {
                        slash_from_target += *target;
                        *value += sp_std::mem::replace(target, Zero::zero());
                    }

                    *total_remaining = total_remaining.saturating_sub(slash_from_target);
                    *value -= slash_from_target;
                }
            };

        slash_out_of(total, active, &mut value);

        let i = self
            .unlocking
            .iter_mut()
            .map(|chunk| {
                slash_out_of(total, &mut chunk.value, &mut value);
                chunk.value
            })
            .take_while(|value| value.is_zero()) // take all fully-consumed chunks out.
            .count();

        // kill all drained chunks.
        let _ = self.unlocking.drain(..i);

        pre_total.saturating_sub(*total)
    }
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Exposure<AccountId, Balance: HasCompact> {
    /// The total balance backing this validator.
    #[codec(compact)]
    pub total: Balance,
    /// The validator's own stash that is exposed.
    #[codec(compact)]
    pub own: Balance,
    /// The delegators that are exposed.
    pub others: Vec<AccountId>,
}

impl<AccountId, Balance: Default + HasCompact> Default for Exposure<AccountId, Balance> {
    fn default() -> Self {
        Self {
            total: Default::default(),
            own: Default::default(),
            others: vec![],
        }
    }
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct UnappliedSlash<AccountId, Balance: HasCompact> {
    /// The stash ID of the offending validator.
    validator: AccountId,
    /// The validator's own slash.
    own: Balance,
    /// All other slashed delegators.
    others: Vec<AccountId>,
    /// Reporters of the offence; bounty payout recipients.
    reporters: Vec<AccountId>,
    /// The amount of payout.
    payout: Balance,
}

impl<AccountId, Balance: HasCompact + Zero> UnappliedSlash<AccountId, Balance> {
    /// Initializes the default object using the given `validator`.
    pub fn default_from(validator: AccountId) -> Self {
        Self {
            validator,
            own: Zero::zero(),
            others: vec![],
            reporters: vec![],
            payout: Zero::zero(),
        }
    }
}

/// Indicate how an election round was computed.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ElectionCompute {
    /// Result was forcefully computed on chain at the end of the session.
    OnChain,
    /// Result was submitted and accepted to the chain via a signed transaction.
    Signed,
    /// Result was submitted and accepted to the chain via an unsigned transaction (by an
    /// authority).
    Unsigned,
}

/// The result of an election round.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
    /// Flat list of validators who have been elected.
    elected_stashes: Vec<AccountId>,
    /// Flat list of new exposures, to be updated in the [`Exposure`] storage.
    exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
    /// Type of the result. This is kept on chain only to track and report the best score's
    /// submission type. An optimisation could remove this.
    compute: ElectionCompute,
}

/// The status of the upcoming (offchain) election.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ElectionStatus<BlockNumber> {
    /// Nothing has and will happen for now. submission window is not open.
    Closed,
    /// The submission window has been open since the contained block number.
    Open(BlockNumber),
}

impl<BlockNumber: PartialEq> ElectionStatus<BlockNumber> {
    pub fn is_open_at(&self, n: BlockNumber) -> bool {
        *self == Self::Open(n)
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::Closed => true,
            _ => false,
        }
    }

    pub fn is_open(&self) -> bool {
        !self.is_closed()
    }
}

impl<BlockNumber> Default for ElectionStatus<BlockNumber> {
    fn default() -> Self {
        Self::Closed
    }
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Config`
pub trait SessionInterface<AccountId>: frame_system::Config {
    /// Disable a given validator by stash ID.
    ///
    /// Returns `true` if new era should be forced at the end of this session.
    /// This allows preventing a situation where there is too many validators
    /// disabled and block production stalls.
    fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
    /// Get the validators from session.
    fn validators() -> Vec<AccountId>;
    /// Prune historical session tries up to but not including the given index.
    fn prune_historical_up_to(up_to: SessionIndex);
}

impl<T: Config> SessionInterface<<T as frame_system::Config>::AccountId> for T
where
    T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
    T: pallet_session::historical::Config<
        FullIdentification = Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        FullIdentificationOf = ExposureOf<T>,
    >,
    T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Config>::AccountId>,
    T::SessionManager: pallet_session::SessionManager<<T as frame_system::Config>::AccountId>,
    T::ValidatorIdOf: Convert<
        <T as frame_system::Config>::AccountId,
        Option<<T as frame_system::Config>::AccountId>,
    >,
{
    fn disable_validator(validator: &<T as frame_system::Config>::AccountId) -> Result<bool, ()> {
        Ok(<pallet_session::Pallet<T>>::disable(validator))
    }

    fn validators() -> Vec<<T as frame_system::Config>::AccountId> {
        <pallet_session::Pallet<T>>::validators()
    }

    fn prune_historical_up_to(up_to: SessionIndex) {
        <pallet_session::historical::Pallet<T>>::prune_up_to(up_to);
    }
}

#[derive(Decode, Encode, TypeInfo)]
pub struct DelegatorData<AccountId> {
    // delegator itself
    pub delegator: AccountId,
    // current delegated validators
    pub delegated_validators: Vec<AccountId>,
    // unrewarded since which era
    pub unrewarded_since: Option<EraIndex>,
    // currently delegating or not
    pub delegating: bool,
}

impl<AccountId: Decode> Default for DelegatorData<AccountId> {
    fn default() -> Self {
        Self {
            delegator: AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
                .expect("nodes should have a valid account id"),
            delegated_validators: Default::default(),
            unrewarded_since: Default::default(),
            delegating: Default::default(),
        }
    }
}

#[derive(Decode, Encode, TypeInfo)]
pub struct ValidatorData<AccountId: Ord> {
    pub delegators: BTreeSet<AccountId>,
    pub elected_era: EraIndex,
}

impl<AccountId: Ord> Default for ValidatorData<AccountId> {
    fn default() -> Self {
        ValidatorData {
            delegators: BTreeSet::new(),
            elected_era: Default::default(),
        }
    }
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Forcing {
    /// Not forcing anything - just let whatever happen.
    NotForcing,
    /// Force a new era, then reset to `NotForcing` as soon as it is done.
    ForceNew,
    /// Avoid a new era indefinitely.
    ForceNone,
    /// Force a new era at the end of all sessions indefinitely.
    ForceAlways,
}

impl Default for Forcing {
    fn default() -> Self {
        Forcing::NotForcing
    }
}

// A value placed in storage that represents the current version of the Staking storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
// This should match directly with the semantic versions of the Rust crate.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
enum Releases {
    V1_0_0Ancient,
    V2_0_0,
    V3_0_0,
    V4_0_0,
    V5_0_0,
    V6_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V6_0_0
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
        /// Number of blocks per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        /// The staking balance.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        /// CreditInterface of credit pallet
        type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;

        /// OperationInterface of operation pallet
        type OperationInterface: OperationInterface<Self::AccountId, BalanceOf<Self>>;

        /// NodeInterface of deeper-node pallet
        type NodeInterface: NodeInterface<Self::AccountId, Self::BlockNumber>;

        /// max delegates can be selected by one delegator
        type MaxDelegates: Get<usize>;

        /// Time used for computing era duration.
        ///
        /// It is guaranteed to start being called from the first `on_finalize`. Thus value at genesis
        /// is not used.
        type UnixTime: UnixTime;

        type NumberToCurrency: Convert<u128, BalanceOf<Self>>;

        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Handler for the unbalanced reduction when slashing a staker.
        type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Number of sessions per era.
        type SessionsPerEra: Get<SessionIndex>;

        /// Number of eras that staked funds must remain bonded for.
        type BondingDuration: Get<EraIndex>;

        /// Number of eras that slashes are deferred by, after computation.
        ///
        /// This should be less than the bonding duration. Set to 0 if slashes
        /// should be applied immediately, without opportunity for intervention.
        type SlashDeferDuration: Get<EraIndex>;

        /// The origin which can cancel a deferred slash. Root can always do this.
        type SlashCancelOrigin: EnsureOrigin<Self::Origin>;

        /// Interface for interacting with a session module.
        type SessionInterface: self::SessionInterface<Self::AccountId>;

        /// The overarching call type.
        type Call: Dispatchable + From<Call<Self>> + IsSubType<Call<Self>> + Clone;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        type TotalMiningReward: Get<u128>;

        type ExistentialDeposit: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::type_value]
    pub(crate) fn HistoryDepthOnEmpty() -> u32 {
        84u32
    }

    /// Number of eras to keep in history.
    ///
    /// Information is kept for eras in `[current_era - history_depth; current_era]`.
    ///
    /// Must be more than the number of eras delayed by session otherwise. I.e. active era must
    /// always be in history. I.e. `active_era > current_era - history_depth` must be
    /// guaranteed.

    #[pallet::storage]
    #[pallet::getter(fn history_depth)]
    pub(crate) type HistoryDepth<T> = StorageValue<_, u32, ValueQuery, HistoryDepthOnEmpty>;

    /// The ideal number of staking participants.
    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn era_validator_reward)]
    pub type EraValidatorReward<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Minimum number of staking participants before emergency conditions are imposed.
    #[pallet::storage]
    #[pallet::getter(fn minimum_validator_count)]
    pub type MinimumValidatorCount<T> = StorageValue<_, u32, ValueQuery>;

    /// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
    /// easy to initialize and the performance hit is minimal (we expect no more than four
    /// invulnerables) and restricted to testnets.
    #[pallet::storage]
    #[pallet::getter(fn invulnerables)]
    pub type Invulnerables<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// Validator WhiteList
    #[pallet::storage]
    #[pallet::getter(fn validator_whitelist)]
    pub type ValidatorWhiteList<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// Map from all locked "stash" accounts to the controller account.
    #[pallet::storage]
    #[pallet::getter(fn bonded)]
    pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::AccountId>;

    /// Map from all (unlocked) "controller" accounts to the info regarding the staking.
    #[pallet::storage]
    #[pallet::getter(fn ledger)]
    pub type Ledger<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, StakingLedger<T::AccountId, BalanceOf<T>>>;

    /// Where the reward payment should be made. Keyed by stash.
    #[pallet::storage]
    #[pallet::getter(fn payee)]
    pub type Payee<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, RewardDestination<T::AccountId>, ValueQuery>;

    /// The map from (wannabe) validator stash key to the preferences of that validator.
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, ValidatorPrefs, ValueQuery>;
    /// The current era index.
    ///
    /// This is the latest planned era, depending on how the Session pallet queues the validator
    /// set, it might be active or not.
    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex>;

    /// The active era information, it holds index and start.
    ///
    /// The active era is the era being currently rewarded. Validator set of this era must be
    /// equal to [`SessionInterface::validators`].
    #[pallet::storage]
    #[pallet::getter(fn active_era)]
    pub type ActiveEra<T> = StorageValue<_, ActiveEraInfo>;

    /// The session index at which the era start for the last `HISTORY_DEPTH` eras.
    ///
    /// Note: This tracks the starting session (i.e. session index when era start being active)
    /// for the eras in `[CurrentEra - HISTORY_DEPTH, CurrentEra]`.
    #[pallet::storage]
    #[pallet::getter(fn eras_start_session_index)]
    pub type ErasStartSessionIndex<T> = StorageMap<_, Twox64Concat, EraIndex, SessionIndex>;

    /// Exposure of validator at era.
    ///
    /// This is keyed first by the era index to allow bulk deletion and then the stash account.
    ///
    /// Is it removed after `HISTORY_DEPTH` eras.
    /// If stakers hasn't been set or has been removed then empty exposure is returned.

    #[pallet::storage]
    #[pallet::getter(fn eras_stakers)]
    pub type ErasStakers<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        Exposure<T::AccountId, BalanceOf<T>>,
        ValueQuery,
    >;

    /// Similar to `ErasStakers`, this holds the preferences of validators.
    ///
    /// This is keyed first by the era index to allow bulk deletion and then the stash account.
    ///
    /// Is it removed after `HISTORY_DEPTH` eras.
    // If prefs hasn't been set or has been removed then 0 commission is returned.
    #[pallet::storage]
    #[pallet::getter(fn eras_validator_prefs)]
    pub type ErasValidatorPrefs<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        ValidatorPrefs,
        ValueQuery,
    >;

    /// Rewards for the last `HISTORY_DEPTH` eras.
    /// If reward hasn't been set or has been removed then 0 reward is returned.
    #[pallet::storage]
    #[pallet::getter(fn eras_reward_points)]
    pub type ErasRewardPoints<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, EraRewardPoints<T::AccountId>, ValueQuery>;

    /// The total amount staked for the last `HISTORY_DEPTH` eras.
    /// If total hasn't been set or has been removed then 0 stake is returned.
    #[pallet::storage]
    #[pallet::getter(fn eras_total_stake)]
    pub type ErasTotalStake<T: Config> =
        StorageMap<_, Twox64Concat, EraIndex, BalanceOf<T>, ValueQuery>;

    /// Mode of era forcing.
    #[pallet::storage]
    #[pallet::getter(fn force_era)]
    pub type ForceEra<T> = StorageValue<_, Forcing, ValueQuery>;

    /// The percentage of the slash that is distributed to reporters.
    ///
    /// The rest of the slashed value is handled by the `Slash`.
    #[pallet::storage]
    #[pallet::getter(fn slash_reward_fraction)]
    pub type SlashRewardFraction<T> = StorageValue<_, Perbill, ValueQuery>;

    /// The amount of currency given to reporters of a slash event which was
    /// canceled by extraordinary circumstances (e.g. governance).
    #[pallet::storage]
    #[pallet::getter(fn canceled_payout)]
    pub type CanceledSlashPayout<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// All unapplied slashes that are queued for later.
    #[pallet::storage]
    pub type UnappliedSlashes<T: Config> = StorageMap<
        _,
        Twox64Concat,
        EraIndex,
        Vec<UnappliedSlash<T::AccountId, BalanceOf<T>>>,
        ValueQuery,
    >;

    /// A mapping from still-bonded eras to the first session index of that era.
    ///
    /// Must contains information for eras for the range:
    /// `[active_era - bounding_duration; active_era]`
    #[pallet::storage]
    pub(crate) type BondedEras<T: Config> =
        StorageValue<_, Vec<(EraIndex, SessionIndex)>, ValueQuery>;

    /// All slashing events on validators, mapped by era to the highest slash proportion
    /// and slash value of the era.
    #[pallet::storage]
    pub(crate) type ValidatorSlashInEra<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        EraIndex,
        Twox64Concat,
        T::AccountId,
        (Perbill, BalanceOf<T>),
    >;

    /// Slashing spans for stash accounts.
    #[pallet::storage]
    pub(crate) type SlashingSpans<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, slashing::SlashingSpans>;
    /// Records information about the maximum slash of a stash within a slashing span,
    /// as well as how much reward has been paid out.
    #[pallet::storage]
    pub(crate) type SpanSlash<T: Config> = StorageMap<
        _,
        Twox64Concat,
        (T::AccountId, slashing::SpanIndex),
        slashing::SpanRecord<BalanceOf<T>>,
        ValueQuery,
    >;

    /// The earliest era for which we have a pending, unapplied slash.
    #[pallet::storage]
    pub(crate) type EarliestUnappliedSlash<T> = StorageValue<_, EraIndex>;

    /// Flag to control the execution of the offchain election. When `Open(_)`, we accept
    /// solutions to be submitted.
    #[pallet::storage]
    #[pallet::getter(fn era_election_status)]
    pub type EraElectionStatus<T: Config> =
        StorageValue<_, ElectionStatus<T::BlockNumber>, ValueQuery>;

    /// True if the current **planned** session is final. Note that this does not take era
    /// forcing into account.
    #[pallet::storage]
    #[pallet::getter(fn is_current_session_final)]
    pub type IsCurrentSessionFinal<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn remainder_mining_reward)]
    pub type RemainderMiningReward<T> = StorageValue<_, u128>;

    /// validator -> ValidatorData
    #[pallet::storage]
    #[pallet::getter(fn candidate_validators)]
    pub(crate) type CandidateValidators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorData<T::AccountId>, ValueQuery>;

    /// delegator -> DelegatorData
    #[pallet::storage]
    #[pallet::getter(fn delegators)]
    pub(crate) type Delegators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DelegatorData<T::AccountId>, ValueQuery>;

    /// active delegator count
    #[pallet::storage]
    #[pallet::getter(fn active_delegator_count)]
    pub(crate) type ActiveDelegatorCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn delegator_count)]
    pub(crate) type DelegatorCount<T> = StorageValue<_, u32, ValueQuery>;

    /// delegators key prefix
    #[pallet::storage]
    #[pallet::getter(fn delegators_key_prefix)]
    pub(crate) type DelegatorsKeyPrefix<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    /// delegators last key
    #[pallet::storage]
    #[pallet::getter(fn delegators_last_key)]
    pub(crate) type DelegatorsLastKey<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn delegator_payouts_per_block)]
    pub(crate) type DelegatorPayoutsPerBlock<T> = StorageValue<_, u32, ValueQuery>;

    /// EraIndex -> validators
    #[pallet::storage]
    #[pallet::getter(fn eras_validators)]
    pub(crate) type ErasValidators<T: Config> =
        StorageMap<_, Blake2_128Concat, EraIndex, Vec<T::AccountId>, ValueQuery>;

    /// reward of delegator
    #[pallet::storage]
    #[pallet::getter(fn reward)]
    pub type Reward<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, RewardData<BalanceOf<T>>>;
    /// True if network has been upgraded to this version.
    /// Storage version of the pallet.
    ///
    /// This is set to v5.0.0 for new networks.
    // StorageVersion build(|_: &GenesisConfig<T>| Releases::V6_0_0): Releases;
    #[pallet::storage]
    pub(crate) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn delegator_balances)]
    pub(crate) type DelegatorBalances<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub history_depth: u32,
        pub validator_count: u32,
        pub minimum_validator_count: u32,
        pub invulnerables: Vec<T::AccountId>,
        pub force_era: Forcing,
        pub era_validator_reward: BalanceOf<T>,
        pub slash_reward_fraction: Perbill,
        pub stakers: Vec<(
            T::AccountId,
            T::AccountId,
            BalanceOf<T>,
            crate::StakerStatus,
        )>,
        pub delegations: Vec<(T::AccountId, Vec<T::AccountId>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                history_depth: 84u32,
                validator_count: Default::default(),
                minimum_validator_count: Default::default(),
                force_era: Default::default(),
                era_validator_reward: Default::default(),
                invulnerables: Default::default(),
                slash_reward_fraction: Default::default(),
                stakers: Default::default(),
                delegations: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            HistoryDepth::<T>::put(self.history_depth);
            ValidatorCount::<T>::put(self.validator_count);
            MinimumValidatorCount::<T>::put(self.minimum_validator_count);
            Invulnerables::<T>::put(&self.invulnerables);
            ForceEra::<T>::put(self.force_era);
            EraValidatorReward::<T>::put(self.era_validator_reward);
            SlashRewardFraction::<T>::put(self.slash_reward_fraction);
            StorageVersion::<T>::put(Releases::V5_0_0);
            for &(ref stash, ref controller, balance, ref status) in &self.stakers {
                assert!(
                    T::Currency::free_balance(&stash) >= balance,
                    "Stash does not have enough balance to bond."
                );
                let _ = <Pallet<T>>::bond(
                    T::Origin::from(Some(stash.clone()).into()),
                    T::Lookup::unlookup(controller.clone()),
                    balance,
                    RewardDestination::Staked,
                );
                let _ = match status {
                    StakerStatus::Validator => <Pallet<T>>::validate(
                        T::Origin::from(Some(controller.clone()).into()),
                        Default::default(),
                    ),
                    _ => Ok(()),
                };
            }
            for &(ref delegator, ref validators) in &self.delegations {
                <Pallet<T>>::delegate(
                    T::Origin::from(Some(delegator.clone()).into()),
                    (*validators).clone(),
                )
                .unwrap();
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::staking_delegate())]
        pub fn staking_delegate(origin: OriginFor<T>, dst_level: u8) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let credit_balances = T::CreditInterface::get_credit_balance(&who);
            let len = credit_balances.len();

            ensure!(len > 0, Error::<T>::NotAllowUpdateCrdit);
            ensure!(
                usize::from(dst_level) < len,
                Error::<T>::TargetLevelNotCorrect
            );

            let credit_score = T::CreditInterface::get_credit_score(&who);
            let (need_balance, score_gap) = {
                if let Some(credit_score) = credit_score {
                    let cur_level = T::CreditInterface::get_credit_level(credit_score).into();
                    ensure!(dst_level > cur_level, Error::<T>::TargetLevelLow);
                    let need_balance = credit_balances[dst_level as usize]
                        .saturating_sub(credit_balances[cur_level as usize]);
                    let score_gap = T::CreditInterface::get_credit_gap(dst_level, cur_level);
                    (need_balance, score_gap)
                } else {
                    let need_balance = credit_balances[dst_level as usize];
                    let score_gap = T::CreditInterface::get_credit_gap(dst_level, 0);
                    (need_balance, score_gap)
                }
            };

            T::Currency::transfer(
                &who,
                &Self::account_id(),
                need_balance,
                ExistenceRequirement::KeepAlive,
            )?;
            DelegatorBalances::<T>::mutate(&who, |balance| {
                *balance = balance.saturating_add(need_balance)
            });
            T::CreditInterface::add_or_update_credit(who.clone(), score_gap);
            Self::delegate_any(who.clone())?;
            Self::deposit_event(Event::<T>::StakingDelegate(who, dst_level));
            Ok(())
        }

        /// Take the origin account as a stash and lock up `value` of its balance. `controller` will
        /// be the account that controls it.
        ///
        /// `value` must be more than the `minimum_balance` specified by `T::Currency`.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash account.
        ///
        /// Emits `Bonded`.
        ///
        /// # <weight>
        /// - Independent of the arguments. Moderate complexity.
        /// - O(1).
        /// - Three extra DB entries.
        ///
        /// NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned
        /// unless the `origin` falls below _existential deposit_ and gets removed as dust.
        /// ------------------
        /// Weight: O(1)
        /// DB Weight:
        /// - Read: Bonded, Ledger, [Origin Account], Current Era, History Depth, Locks
        /// - Write: Bonded, Payee, [Origin Account], Locks, Ledger
        /// # </weight>
        #[pallet::weight(T::WeightInfo::bond())]
        pub fn bond(
            origin: OriginFor<T>,
            controller: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            let stash = ensure_signed(origin)?;

            if <Bonded<T>>::contains_key(&stash) {
                Err(Error::<T>::AlreadyBonded)?
            }

            let controller = T::Lookup::lookup(controller)?;

            if <Ledger<T>>::contains_key(&controller) {
                Err(Error::<T>::AlreadyPaired)?
            }

            // reject a bond which is considered to be _dust_.
            if value < T::Currency::minimum_balance() {
                Err(Error::<T>::InsufficientValue)?
            }

            frame_system::Pallet::<T>::inc_consumers(&stash).map_err(|_| Error::<T>::BadState)?;

            // You're auto-bonded forever, here. We might improve this by only bonding when
            // you actually validate and remove once you unbond __everything__.
            <Bonded<T>>::insert(&stash, &controller);
            <Payee<T>>::insert(&stash, payee);

            let current_era = CurrentEra::<T>::get().unwrap_or(0);
            let history_depth = Self::history_depth();
            let last_reward_era = current_era.saturating_sub(history_depth);

            let stash_balance = T::Currency::free_balance(&stash);
            let value = value.min(stash_balance);
            Self::deposit_event(Event::<T>::Bonded(stash.clone(), value));
            let item = StakingLedger {
                stash,
                total: value,
                active: value,
                unlocking: vec![],
                claimed_rewards: (last_reward_era..current_era).collect(),
            };
            Self::update_ledger(&controller, &item);
            Ok(())
        }

        /// Add some extra amount that have appeared in the stash `free_balance` into the balance up
        /// for staking.
        ///
        /// Use this if there are additional funds in your stash account that you wish to bond.
        /// Unlike [`bond`] or [`unbond`] this function does not impose any limitation on the amount
        /// that can be added.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash, not the controller and
        /// it can be only called when [`EraElectionStatus`] is `Closed`.
        ///
        /// Emits `Bonded`.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - O(1).
        /// - One DB entry.
        /// ------------
        /// DB Weight:
        /// - Read: Era Election Status, Bonded, Ledger, [Origin Account], Locks
        /// - Write: [Origin Account], Locks, Ledger
        /// # </weight>
        #[pallet::weight(T::WeightInfo::bond_extra())]
        pub fn bond_extra(
            origin: OriginFor<T>,
            #[pallet::compact] max_additional: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let stash = ensure_signed(origin)?;

            let controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;

            let stash_balance = T::Currency::free_balance(&stash);
            if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
                let extra = extra.min(max_additional);
                ledger.total += extra;
                ledger.active += extra;
                // last check: the new active amount of ledger must be more than ED.
                ensure!(
                    ledger.active >= T::Currency::minimum_balance(),
                    Error::<T>::InsufficientValue
                );

                Self::deposit_event(Event::<T>::Bonded(stash, extra));
                Self::update_ledger(&controller, &ledger);
            }
            Ok(())
        }

        /// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
        /// period ends. If this leaves an amount actively bonded less than
        /// T::Currency::minimum_balance(), then it is increased to the full amount.
        ///
        /// Once the unlock period is done, you can call `withdraw_unbonded` to actually move
        /// the funds out of management ready for transfer.
        ///
        /// No more than a limited number of unlocking chunks (see `MAX_UNLOCKING_CHUNKS`)
        /// can co-exists at the same time. In that case, [`Call::withdraw_unbonded`] need
        /// to be called first to remove some of the chunks (if possible).
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        /// And, it can be only called when [`EraElectionStatus`] is `Closed`.
        ///
        /// Emits `Unbonded`.
        ///
        /// See also [`Call::withdraw_unbonded`].
        ///
        /// # <weight>
        /// - Independent of the arguments. Limited but potentially exploitable complexity.
        /// - Contains a limited number of reads.
        /// - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
        ///   will cause a new entry to be inserted into a vector (`Ledger.unlocking`) kept in storage.
        ///   The only way to clean the aforementioned storage item is also user-controlled via
        ///   `withdraw_unbonded`.
        /// - One DB entry.
        /// ----------
        /// Weight: O(1)
        /// DB Weight:
        /// - Read: EraElectionStatus, Ledger, CurrentEra, Locks, BalanceOf Stash,
        /// - Write: Locks, Ledger, BalanceOf Stash,
        /// </weight>
        #[pallet::weight(T::WeightInfo::unbond())]
        pub fn unbond(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            ensure!(
                ledger.unlocking.len() < MAX_UNLOCKING_CHUNKS,
                Error::<T>::NoMoreChunks,
            );

            let mut value = value.min(ledger.active);

            if !value.is_zero() {
                ledger.active -= value;

                // Avoid there being a dust balance left in the staking system.
                if ledger.active < T::Currency::minimum_balance() {
                    value += ledger.active;
                    ledger.active = Zero::zero();
                }

                // Note: in case there is no current era it is fine to bond one era more.
                let era = Self::current_era().unwrap_or(0) + T::BondingDuration::get();
                ledger.unlocking.push(UnlockChunk { value, era });
                Self::update_ledger(&controller, &ledger);
                Self::deposit_event(Event::<T>::Unbonded(ledger.stash, value));
            }
            Ok(())
        }

        /// Remove any unlocked chunks from the `unlocking` queue from our management.
        ///
        /// This essentially frees up that balance to be used by the stash account to do
        /// whatever it wants.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        /// And, it can be only called when [`EraElectionStatus`] is `Closed`.
        ///
        /// Emits `Withdrawn`.
        ///
        /// See also [`Call::unbond`].
        ///
        /// # <weight>
        /// - Could be dependent on the `origin` argument and how much `unlocking` chunks exist.
        ///  It implies `consolidate_unlocked` which loops over `Ledger.unlocking`, which is
        ///  indirectly user-controlled. See [`unbond`] for more detail.
        /// - Contains a limited number of reads, yet the size of which could be large based on `ledger`.
        /// - Writes are limited to the `origin` account key.
        /// ---------------
        /// Complexity O(S) where S is the number of slashing spans to remove
        /// Update:
        /// - Reads: EraElectionStatus, Ledger, Current Era, Locks, [Origin Account]
        /// - Writes: [Origin Account], Locks, Ledger
        /// Kill:
        /// - Reads: EraElectionStatus, Ledger, Current Era, Bonded, Slashing Spans, [Origin
        ///   Account], Locks, BalanceOf stash
        /// - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators,
        ///   [Origin Account], Locks, BalanceOf stash.
        /// - Writes Each: SpanSlash * S
        /// NOTE: Weight annotation is the kill scenario, we refund otherwise.
        /// # </weight>
        #[pallet::weight(T::WeightInfo::withdraw_unbonded_kill(*num_slashing_spans))]
        pub fn withdraw_unbonded(
            origin: OriginFor<T>,
            num_slashing_spans: u32,
        ) -> DispatchResultWithPostInfo {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let (stash, old_total) = (ledger.stash.clone(), ledger.total);
            if let Some(current_era) = Self::current_era() {
                ledger = ledger.consolidate_unlocked(current_era)
            }

            let post_info_weight =
                if ledger.unlocking.is_empty() && ledger.active <= T::Currency::minimum_balance() {
                    // This account must have called `unbond()` with some value that caused the active
                    // portion to fall below existential deposit + will have no more unlocking chunks
                    // left. We can now safely remove all staking-related information.
                    Self::kill_stash(&stash, num_slashing_spans)?;
                    // remove the lock.
                    T::Currency::remove_lock(STAKING_ID, &stash);
                    // This is worst case scenario, so we use the full weight and return None
                    None
                } else {
                    // This was the consequence of a partial unbond. just update the ledger and move on.
                    Self::update_ledger(&controller, &ledger);

                    // This is only an update, so we use less overall weight.
                    Some(T::WeightInfo::withdraw_unbonded_update(num_slashing_spans))
                };

            // `old_total` should never be less than the new total because
            // `consolidate_unlocked` strictly subtracts balance.
            if ledger.total < old_total {
                // Already checked that this won't overflow by entry condition.
                let value = old_total - ledger.total;
                Self::deposit_event(Event::<T>::Withdrawn(stash, value));
            }

            Ok(post_info_weight.into())
        }

        /// Declare the desire to validate for the origin controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        /// And, it can be only called when [`EraElectionStatus`] is `Closed`.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// -----------
        /// Weight: O(1)
        /// DB Weight:
        /// - Read: Era Election Status, Ledger
        /// - Write: Validators
        /// # </weight>
        #[pallet::weight(T::WeightInfo::validate())]
        pub fn validate(origin: OriginFor<T>, prefs: ValidatorPrefs) -> DispatchResult {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let stash = &ledger.stash;
            <Validators<T>>::insert(stash, prefs);
            Ok(())
        }

        /// Declare no desire to either validate.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        /// And, it can be only called when [`EraElectionStatus`] is `Closed`.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains one read.
        /// - Writes are limited to the `origin` account key.
        /// --------
        /// Weight: O(1)
        /// DB Weight:
        /// - Read: EraElectionStatus, Ledger
        /// - Write: Validators
        /// # </weight>
        #[pallet::weight(T::WeightInfo::chill())]
        pub fn chill(origin: OriginFor<T>) -> DispatchResult {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            Self::chill_stash(&ledger.stash);
            Ok(())
        }

        /// (Re-)set the payment target for a controller.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// ---------
        /// - Weight: O(1)
        /// - DB Weight:
        ///     - Read: Ledger
        ///     - Write: Payee
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_payee())]
        pub fn set_payee(
            origin: OriginFor<T>,
            payee: RewardDestination<T::AccountId>,
        ) -> DispatchResult {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let stash = &ledger.stash;
            <Payee<T>>::insert(stash, payee);
            Ok(())
        }

        /// (Re-)set the controller of a stash.
        ///
        /// Effects will be felt at the beginning of the next era.
        ///
        /// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
        ///
        /// # <weight>
        /// - Independent of the arguments. Insignificant complexity.
        /// - Contains a limited number of reads.
        /// - Writes are limited to the `origin` account key.
        /// ----------
        /// Weight: O(1)
        /// DB Weight:
        /// - Read: Bonded, Ledger New Controller, Ledger Old Controller
        /// - Write: Bonded, Ledger New Controller, Ledger Old Controller
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_controller())]
        pub fn set_controller(
            origin: OriginFor<T>,
            controller: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            let stash = ensure_signed(origin)?;
            let old_controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
            let controller = T::Lookup::lookup(controller)?;
            if <Ledger<T>>::contains_key(&controller) {
                Err(Error::<T>::AlreadyPaired)?
            }
            if controller != old_controller {
                <Bonded<T>>::insert(&stash, &controller);
                if let Some(l) = <Ledger<T>>::take(&old_controller) {
                    <Ledger<T>>::insert(&controller, l);
                }
            }
            Ok(())
        }

        /// Sets the validator reward per era.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Weight: O(1)
        /// Write: EraValidatorReward
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_era_validator_reward())]
        pub fn set_era_validator_reward(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            EraValidatorReward::<T>::put(value);
            Ok(())
        }

        /// Sets the ideal number of validators.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Weight: O(1)
        /// Write: Validator Count
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_validator_count())]
        pub fn set_validator_count(
            origin: OriginFor<T>,
            #[pallet::compact] new: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ValidatorCount::<T>::put(new);
            Ok(())
        }

        /// Increments the ideal number of validators.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Same as [`set_validator_count`].
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_validator_count())]
        pub fn increase_validator_count(
            origin: OriginFor<T>,
            #[pallet::compact] additional: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            ValidatorCount::<T>::mutate(|n| *n += additional);
            Ok(())
        }

        /// Scale up the ideal number of validators by a factor.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Same as [`set_validator_count`].
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_validator_count())]
        pub fn scale_validator_count(origin: OriginFor<T>, factor: Percent) -> DispatchResult {
            ensure_root(origin)?;
            ValidatorCount::<T>::mutate(|n| *n += factor * *n);
            Ok(())
        }

        /// Force there to be no new eras indefinitely.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - No arguments.
        /// - Weight: O(1)
        /// - Write: ForceEra
        /// # </weight>
        #[pallet::weight(T::WeightInfo::force_no_eras())]
        pub fn force_no_eras(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            ForceEra::<T>::put(Forcing::ForceNone);
            Ok(())
        }

        /// Force there to be a new era at the end of the next session. After this, it will be
        /// reset to normal (non-forced) behaviour.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - No arguments.
        /// - Weight: O(1)
        /// - Write ForceEra
        /// # </weight>
        #[pallet::weight(T::WeightInfo::force_new_era())]
        pub fn force_new_era(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            ForceEra::<T>::put(Forcing::ForceNew);
            Ok(())
        }

        /// Set the validators who cannot be slashed (if any).
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - O(V)
        /// - Write: Invulnerables
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_invulnerables(invulnerables.len() as u32))]
        pub fn set_invulnerables(
            origin: OriginFor<T>,
            invulnerables: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <Invulnerables<T>>::put(invulnerables);
            Ok(())
        }

        /// Set the validator white list.
        /// null , any validators can come up for election; not null, only validators in whitelist can come up for election
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - O(V)
        /// - Write: ValidatorWhiteList
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_validator_whitelist(whitelist.len() as u32))]
        pub fn set_validator_whitelist(
            origin: OriginFor<T>,
            whitelist: Vec<T::AccountId>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            <ValidatorWhiteList<T>>::put(whitelist.clone());
            Self::deposit_event(Event::<T>::SetValidatorWhitelist(whitelist));
            Ok(())
        }

        /// Force a current staker to become completely unstaked, immediately.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// O(S) where S is the number of slashing spans to be removed
        /// Reads: Bonded, Slashing Spans, Account, Locks
        /// Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Account, Locks
        /// Writes Each: SpanSlash * S
        /// # </weight>
        #[pallet::weight(T::WeightInfo::force_unstake(*num_slashing_spans))]
        pub fn force_unstake(
            origin: OriginFor<T>,
            stash: T::AccountId,
            num_slashing_spans: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // remove all staking-related information.
            Self::kill_stash(&stash, num_slashing_spans)?;

            // remove the lock.
            T::Currency::remove_lock(STAKING_ID, &stash);
            Ok(())
        }

        /// Force there to be a new era at the end of sessions indefinitely.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - Weight: O(1)
        /// - Write: ForceEra
        /// # </weight>
        #[pallet::weight(T::WeightInfo::force_new_era_always())]
        pub fn force_new_era_always(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            ForceEra::<T>::put(Forcing::ForceAlways);
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::increase_mining_reward(1))]
        pub fn increase_mining_reward(
            origin: OriginFor<T>,
            additional_reward: u128,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let remainder = Self::remainder_mining_reward().unwrap_or(T::TotalMiningReward::get());
            RemainderMiningReward::<T>::put(remainder + additional_reward);
            Self::deposit_event(Event::<T>::IncreaseMiningReward(additional_reward));
            Ok(())
        }

        /// Cancel enactment of a deferred slash.
        ///
        /// Can be called by the `T::SlashCancelOrigin`.
        ///
        /// Parameters: era and indices of the slashes for that era to kill.
        ///
        /// # <weight>
        /// Complexity: O(U + S)
        /// with U unapplied slashes weighted with U=1000
        /// and S is the number of slash indices to be canceled.
        /// - Read: Unapplied Slashes
        /// - Write: Unapplied Slashes
        /// # </weight>
        #[pallet::weight(T::WeightInfo::cancel_deferred_slash(slash_indices.len() as u32))]
        pub fn cancel_deferred_slash(
            origin: OriginFor<T>,
            era: EraIndex,
            slash_indices: Vec<u32>,
        ) -> DispatchResult {
            T::SlashCancelOrigin::ensure_origin(origin)?;

            ensure!(!slash_indices.is_empty(), Error::<T>::EmptyTargets);
            ensure!(
                is_sorted_and_unique(&slash_indices),
                Error::<T>::NotSortedAndUnique
            );

            let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);
            let last_item = slash_indices[slash_indices.len() - 1];
            ensure!(
                (last_item as usize) < unapplied.len(),
                Error::<T>::InvalidSlashIndex
            );

            for (removed, index) in slash_indices.into_iter().enumerate() {
                let index = (index as usize) - removed;
                unapplied.remove(index);
            }

            <Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
            Ok(())
        }

        /// Rebond a portion of the stash scheduled to be unlocked.
        ///
        /// The dispatch origin must be signed by the controller, and it can be only called when
        /// [`EraElectionStatus`] is `Closed`.
        ///
        /// # <weight>
        /// - Time complexity: O(L), where L is unlocking chunks
        /// - Bounded by `MAX_UNLOCKING_CHUNKS`.
        /// - Storage changes: Can't increase storage, only decrease it.
        /// ---------------
        /// - DB Weight:
        ///     - Reads: EraElectionStatus, Ledger, Locks, [Origin Account]
        ///     - Writes: [Origin Account], Locks, Ledger
        /// # </weight>
        #[pallet::weight(T::WeightInfo::rebond(MAX_UNLOCKING_CHUNKS as u32))]
        pub fn rebond(
            origin: OriginFor<T>,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            ensure!(!ledger.unlocking.is_empty(), Error::<T>::NoUnlockChunk);

            let ledger = ledger.rebond(value);
            // last check: the new active amount of ledger must be more than ED.
            ensure!(
                ledger.active >= T::Currency::minimum_balance(),
                Error::<T>::InsufficientValue
            );

            Self::update_ledger(&controller, &ledger);
            Ok(Some(
                35 * WEIGHT_PER_MICROS
                    + 50 * WEIGHT_PER_NANOS * (ledger.unlocking.len() as Weight)
                    + T::DbWeight::get().reads_writes(3, 2),
            )
            .into())
        }

        /// Set `HistoryDepth` value. This function will delete any history information
        /// when `HistoryDepth` is reduced.
        ///
        /// Parameters:
        /// - `new_history_depth`: The new history depth you would like to set.
        /// - `era_items_deleted`: The number of items that will be deleted by this dispatch.
        ///    This should report all the storage items that will be deleted by clearing old
        ///    era history. Needed to report an accurate weight for the dispatch. Trusted by
        ///    `Root` to report an accurate number.
        ///
        /// Origin must be root.
        ///
        /// # <weight>
        /// - E: Number of history depths removed, i.e. 10 -> 7 = 3
        /// - Weight: O(E)
        /// - DB Weight:
        ///     - Reads: Current Era, History Depth
        ///     - Writes: History Depth
        ///     - Clear Prefix Each: Era Stakers, EraStakersClipped, ErasValidatorPrefs
        ///     - Writes Each: ErasRewardPoints, ErasTotalStake, ErasStartSessionIndex
        /// # </weight>
        #[pallet::weight(T::WeightInfo::set_history_depth(*_era_items_deleted))]
        pub fn set_history_depth(
            origin: OriginFor<T>,
            #[pallet::compact] new_history_depth: EraIndex,
            #[pallet::compact] _era_items_deleted: u32,
        ) -> DispatchResult {
            ensure_root(origin)?;
            if let Some(current_era) = Self::current_era() {
                HistoryDepth::<T>::mutate(|history_depth| {
                    let last_kept = current_era.checked_sub(*history_depth).unwrap_or(0);
                    let new_last_kept = current_era.checked_sub(new_history_depth).unwrap_or(0);
                    for era_index in last_kept..new_last_kept {
                        Self::clear_era_information(era_index);
                    }
                    *history_depth = new_history_depth
                })
            }
            Ok(())
        }

        /// Remove all data structure concerning a staker/stash once its balance is at the minimum.
        /// This is essentially equivalent to `withdraw_unbonded` except it can be called by anyone
        /// and the target `stash` must have no funds left beyond the ED.
        ///
        /// This can be called from any origin.
        ///
        /// - `stash`: The stash account to reap. Its balance must be zero.
        ///
        /// # <weight>
        /// Complexity: O(S) where S is the number of slashing spans on the account.
        /// DB Weight:
        /// - Reads: Stash Account, Bonded, Slashing Spans, Locks
        /// - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Stash Account, Locks
        /// - Writes Each: SpanSlash * S
        /// # </weight>
        #[pallet::weight(T::WeightInfo::reap_stash(*num_slashing_spans))]
        pub fn reap_stash(
            _origin: OriginFor<T>,
            stash: T::AccountId,
            num_slashing_spans: u32,
        ) -> DispatchResult {
            let at_minimum = T::Currency::total_balance(&stash) == T::Currency::minimum_balance();
            ensure!(at_minimum, Error::<T>::FundedTarget);
            Self::kill_stash(&stash, num_slashing_spans)?;
            T::Currency::remove_lock(STAKING_ID, &stash);
            Ok(())
        }

        /// delegate credit to a set of validators
        #[pallet::weight(T::WeightInfo::delegate(1))]
        pub fn delegate(origin: OriginFor<T>, validators: Vec<T::AccountId>) -> DispatchResult {
            ensure!(
                Self::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let delegator = ensure_signed(origin)?;

            ensure!(
                !<Validators<T>>::contains_key(&delegator),
                Error::<T>::CallNotAllowed
            );

            ensure!(!validators.is_empty(), Error::<T>::NoValidators);
            // remove duplicates
            let validator_set: BTreeSet<T::AccountId> = validators.iter().cloned().collect();
            // check validators size
            ensure!(
                validator_set.len() <= T::MaxDelegates::get() as usize,
                Error::<T>::TooManyValidators
            );
            for validator in &validator_set {
                ensure!(
                    <Validators<T>>::contains_key(&validator),
                    Error::<T>::NotValidator
                );
            }

            Self::do_delegate(delegator, validators)
        }

        /// undelegate credit from the validators
        #[pallet::weight(T::WeightInfo::undelegate())]
        pub fn undelegate(origin: OriginFor<T>) -> DispatchResult {
            let delegator = ensure_signed(origin)?;
            ensure!(
                <Delegators<T>>::contains_key(&delegator),
                Error::<T>::NotDelegator
            );

            Self::_undelegate(&delegator);
            Self::deposit_event(Event::<T>::UnDelegated(delegator));
            Ok(())
        }

        #[pallet::weight(T::WeightInfo::undelegate())]
        pub fn force_undelegate(
            origin: OriginFor<T>,
            accounts: Vec<(T::AccountId, u64)>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            for (account, score) in accounts {
                T::CreditInterface::slash_credit(&account, Some(score));
                Self::_undelegate(&account);
                Self::deposit_event(Event::<T>::UnDelegated(account));
            }
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1))]
        pub fn difference_compensation(
            origin: OriginFor<T>,
            delegator: T::AccountId,
            referee_reward: BalanceOf<T>,
            poc_reward: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::OperationInterface::is_payment_address(who),
                Error::<T>::UnauthorizedAccounts
            );
            let remainder_mining_reward = T::NumberToCurrency::convert(
                Self::remainder_mining_reward().unwrap_or(T::TotalMiningReward::get()),
            );

            // update RewardData
            if Reward::<T>::contains_key(&delegator) {
                // 1 read
                Reward::<T>::mutate(&delegator, |data| match data {
                    // 1 write
                    Some(reward_data) => {
                        reward_data.received_referee_reward += referee_reward;
                        reward_data.referee_reward = referee_reward;
                        reward_data.received_pocr_reward += poc_reward;
                        reward_data.poc_reward = poc_reward;
                    }
                    _ => (),
                });
            } else {
                let (total_referee_reward, _) =
                    T::CreditInterface::get_top_referee_reward(&delegator);
                let reward_data = RewardData::<BalanceOf<T>> {
                    total_referee_reward,
                    received_referee_reward: referee_reward,
                    referee_reward: referee_reward,
                    received_pocr_reward: poc_reward,
                    poc_reward: poc_reward,
                };
                Reward::<T>::insert(&delegator, reward_data);
            }
            let reward = cmp::min(remainder_mining_reward, referee_reward + poc_reward);
            ensure!(
                T::OperationInterface::is_single_max_limit(reward),
                Error::<T>::PaymentsExceedingLimits
            );
            let imbalance = T::Currency::deposit_creating(&delegator, reward);
            RemainderMiningReward::<T>::put(
                TryInto::<u128>::try_into(remainder_mining_reward.saturating_sub(reward))
                    .ok()
                    .unwrap(),
            );
            Self::deposit_event(Event::CompensationDelegatorReward(
                delegator.clone(),
                imbalance.peek(),
            ));

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            if StorageVersion::<T>::get() == Releases::V4_0_0 {
                StorageVersion::<T>::put(Releases::V5_0_0);
                return migrations::migrate_to_blockable::<T>();
            } else if StorageVersion::<T>::get() == Releases::V5_0_0 {
                StorageVersion::<T>::put(Releases::V6_0_0);
                let mut to_be_removed = Vec::new();
                for (account, balance) in DelegatorBalances::<T>::iter() {
                    let (slash_score, extra_balance) = Self::calc_extra_balance_credit(&account);
                    if slash_score == 0 {
                        continue;
                    }
                    let _ = T::Currency::transfer(
                        &Self::account_id(),
                        &account,
                        balance.saturating_sub(extra_balance),
                        ExistenceRequirement::AllowDeath,
                    );
                    T::CreditInterface::slash_credit(&account, Some(slash_score));
                    to_be_removed.push(account);
                }

                for account in to_be_removed {
                    DelegatorBalances::<T>::remove(account);
                }
            }
            0
        }

        fn on_initialize(now: T::BlockNumber) -> Weight {
            // payout delegators only after the first era
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            if CurrentEra::<T>::get().unwrap_or(0) > 0 {
                let remainder = now % T::BlocksPerEra::get();
                if remainder == T::BlockNumber::default() {
                    // first block of the era
                    let blocks_per_era = TryInto::<u32>::try_into(T::BlocksPerEra::get())
                        .ok()
                        .unwrap();
                    // figure out how many payouts to make per block, excluding the first and last block of each era
                    let mut delegator_payouts_per_block =
                        Self::delegator_count() / (blocks_per_era - 2);
                    if Self::delegator_count() % (blocks_per_era - 2) > 0 {
                        delegator_payouts_per_block += 1;
                    }
                    DelegatorPayoutsPerBlock::<T>::put(delegator_payouts_per_block);
                    let prefix = Self::get_delegators_prefix_hash();
                    DelegatorsKeyPrefix::<T>::put(prefix.clone());
                    DelegatorsLastKey::<T>::put(prefix);
                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(3, 1));
                } else {
                    weight = weight.saturating_add(Self::pay_delegators());
                }
            }
            let finalize_weight = T::DbWeight::get().reads_writes(1, 1);
            weight.saturating_add(finalize_weight)
        }

        fn on_finalize(_: T::BlockNumber) {
            // Set the start of the first era.
            if let Some(mut active_era) = Self::active_era() {
                if active_era.start.is_none() {
                    let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();
                    active_era.start = Some(now_as_millis_u64);
                    // This write only ever happens once, we don't include it in the weight in general
                    ActiveEra::<T>::put(active_era);
                }
            }
            // `on_finalize` weight is tracked in `on_initialize`
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The era payout has been set; the first balance is the validator-payout; the second is
        /// the remainder from the maximum amount of reward.
        /// \[era_index, validator_payout, remainder\]
        EraPayout(EraIndex, BalanceOf<T>, BalanceOf<T>),
        /// One validator has been slashed by the given amount.
        /// \[validator, amount\]
        Slash(T::AccountId, BalanceOf<T>),
        /// An old slashing report from a prior era was discarded because it could
        /// not be processed. \[session_index\]
        OldSlashingReportDiscarded(SessionIndex),
        /// A new set of stakers was elected with the given \[compute\].
        StakingElection(ElectionCompute),
        /// A new solution for the upcoming election has been stored. \[compute\]
        SolutionStored(ElectionCompute),
        /// An account has bonded this amount. \[stash, amount\]
        ///
        /// NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
        /// it will not be emitted for staking rewards when they are added to stake.
        Bonded(T::AccountId, BalanceOf<T>),
        /// An account has unbonded this amount. \[stash, amount\]
        Unbonded(T::AccountId, BalanceOf<T>),
        /// An account has called `withdraw_unbonded` and removed unbonding chunks worth `BalanceOf<T>`
        /// from the unlocking queue. \[stash, amount\]
        Withdrawn(T::AccountId, BalanceOf<T>),
        /// Delegated to a set of validators
        Delegated(T::AccountId, Vec<T::AccountId>),
        /// Undelegate from a validator
        UnDelegated(T::AccountId),
        /// The delegator  has been rewarded by this amount. \[account_id, amount\]
        DelegatorReward(T::AccountId, BalanceOf<T>),
        /// The delegator  has been compensation_rewarded by this amount. \[account_id, amount\]
        CompensationDelegatorReward(T::AccountId, BalanceOf<T>),
        /// The validator  has been rewarded by this amount. \[account_id, amount\]
        ValidatorReward(T::AccountId, BalanceOf<T>),
        /// Increase mining pool DPR
        IncreaseMiningReward(u128),
        /// Set validator whitelist list
        SetValidatorWhitelist(Vec<T::AccountId>),
        /// On-chain staking, including Genesis and Basic users
        StakingDelegate(T::AccountId, u8),
    }

    /// Error for the staking module.
    #[pallet::error]
    pub enum Error<T> {
        /// Not a controller account.
        NotController,
        /// Not a stash account.
        NotStash,
        /// Stash is already bonded.
        AlreadyBonded,
        /// Controller is already paired.
        AlreadyPaired,
        /// Targets cannot be empty.
        EmptyTargets,
        /// Duplicate index.
        DuplicateIndex,
        /// Slash record index out of bounds.
        InvalidSlashIndex,
        /// Can not bond with value less than minimum balance.
        InsufficientValue,
        /// Can not schedule more unlock chunks.
        NoMoreChunks,
        /// Can not rebond without unlocking chunks.
        NoUnlockChunk,
        /// Attempting to target a stash that still has funds.
        FundedTarget,
        /// Invalid era to reward.
        InvalidEraToReward,
        /// Items are not sorted and unique.
        NotSortedAndUnique,
        /// Rewards for this era have already been claimed for this validator.
        AlreadyClaimed,
        /// The snapshot data of the current window is missing.
        SnapshotUnavailable,
        /// The call is not allowed at the given time due to restrictions of election period.
        CallNotAllowed,
        /// Incorrect previous history depth input provided.
        IncorrectHistoryDepth,
        /// Incorrect number of slashing spans provided.
        IncorrectSlashingSpans,
        /// Internal state has become somehow corrupted and the operation cannot continue.
        BadState,
        /// Have not been delegated to a validator
        NotDelegator,
        /// Credit score of delegator is too low
        CreditTooLow,
        /// Target of delegation is not in candidate validators
        NotValidator,
        /// Select too many candidate validators
        TooManyValidators,
        /// No candidate validator has been selected
        NoValidators,
        /// target credit level not greater than current
        TargetLevelLow,
        /// target credit level greater than all level
        TargetLevelNotCorrect,
        /// Non-authorized accounts
        UnauthorizedAccounts,
        /// Single payment DPR exceeds the maximum limit for authorized accounts
        PaymentsExceedingLimits,
        /// not allow update credit level
        NotAllowUpdateCrdit,
    }
}

pub mod migrations {
    use super::*;

    #[derive(Decode)]
    struct OldValidatorPrefs {
        #[codec(compact)]
        pub commission: Perbill,
    }
    impl OldValidatorPrefs {
        fn upgraded(self) -> ValidatorPrefs {
            ValidatorPrefs {
                commission: self.commission,
                ..Default::default()
            }
        }
    }
    pub fn migrate_to_blockable<T: Config>() -> frame_support::weights::Weight {
        Validators::<T>::translate::<OldValidatorPrefs, _>(|_, p| Some(p.upgraded()));
        ErasValidatorPrefs::<T>::translate::<OldValidatorPrefs, _>(|_, _, p| Some(p.upgraded()));
        T::BlockWeights::get().max_block
    }
}

impl<T: Config> pallet::Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    fn delegate_any(delegator: T::AccountId) -> DispatchResult {
        let validators: Vec<_> = Validators::<T>::iter_keys().collect();
        ensure!(!validators.is_empty(), Error::<T>::NoValidators);
        let num: u64 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let idx = (num as usize) % validators.len();

        Self::do_delegate(delegator, vec![validators[idx].clone()])
    }

    pub fn do_delegate(delegator: T::AccountId, validators: Vec<T::AccountId>) -> DispatchResult {
        let enough_credit = T::CreditInterface::pass_threshold(&delegator);
        ensure!(enough_credit, Error::<T>::CreditTooLow);

        let current_era = T::CreditInterface::get_current_era();
        if <Delegators<T>>::contains_key(&delegator) {
            let old_delegator_data = Self::delegators(&delegator);
            if !old_delegator_data.delegating {
                // the delegator was not delegating
                // the delegator delegates again
                ActiveDelegatorCount::<T>::mutate(|count| *count = count.saturating_add(1));
            }
            let earliest_unrewarded_era = match old_delegator_data.unrewarded_since {
                Some(unrewarded_era) => unrewarded_era,
                None => current_era,
            };
            let delegator_data = DelegatorData {
                delegator: delegator.clone(),
                delegated_validators: validators.clone(),
                unrewarded_since: Some(earliest_unrewarded_era),
                delegating: true,
            };
            <Delegators<T>>::insert(&delegator, delegator_data);

            for validator in &old_delegator_data.delegated_validators {
                <CandidateValidators<T>>::mutate(validator, |v| {
                    v.delegators.remove(&delegator);
                });
                let candidate = Self::candidate_validators(validator);
                if candidate.delegators.is_empty() {
                    <CandidateValidators<T>>::remove(validator);
                }
            }
        } else {
            let delegator_data = DelegatorData {
                delegator: delegator.clone(),
                delegated_validators: validators.clone(),
                unrewarded_since: Some(current_era),
                delegating: true,
            };
            <Delegators<T>>::insert(&delegator, delegator_data);
            ActiveDelegatorCount::<T>::mutate(|count| *count = count.saturating_add(1));
            DelegatorCount::<T>::mutate(|count| *count = count.saturating_add(1));
            //  delegator must has enough credit score,so this init must success
            T::CreditInterface::init_delegator_history(&delegator, current_era);
        };

        for validator in &validators {
            if <CandidateValidators<T>>::contains_key(validator) {
                <CandidateValidators<T>>::mutate(validator, |v| {
                    v.delegators.insert(delegator.clone());
                });
            } else {
                let mut delegators = BTreeSet::new();
                delegators.insert(delegator.clone());
                let elected_era = EraIndex::default();
                <CandidateValidators<T>>::insert(
                    validator,
                    ValidatorData {
                        delegators,
                        elected_era,
                    },
                );
            }
        }

        Self::deposit_event(Event::<T>::Delegated(delegator, validators));
        Ok(())
    }

    /// The total balance that can be slashed from a stash account as of right now.
    pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
        // Weight note: consider making the stake accessible through stash.
        Self::bonded(stash)
            .and_then(Self::ledger)
            .map(|l| l.active)
            .unwrap_or_default()
    }

    /// Update the ledger for a controller.
    ///
    /// This will also update the stash lock.
    fn update_ledger(
        controller: &T::AccountId,
        ledger: &StakingLedger<T::AccountId, BalanceOf<T>>,
    ) {
        T::Currency::set_lock(
            STAKING_ID,
            &ledger.stash,
            ledger.total,
            WithdrawReasons::all(),
        );
        <Ledger<T>>::insert(controller, ledger);
    }

    /// Chill a stash account.
    fn chill_stash(stash: &T::AccountId) {
        <Validators<T>>::remove(stash);
    }

    /// Plan a new session potentially trigger a new era.
    fn new_session(session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        if let Some(current_era) = Self::current_era() {
            // Initial era has been set.

            let current_era_start_session_index = Self::eras_start_session_index(current_era)
                .unwrap_or_else(|| {
                    frame_support::print("Error: start_session_index must be set for current_era");
                    0
                });

            let era_length = session_index
                .checked_sub(current_era_start_session_index)
                .unwrap_or(0); // Must never happen.

            match ForceEra::<T>::get() {
                Forcing::ForceNew => ForceEra::<T>::kill(),
                Forcing::ForceAlways => (),
                Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
                _ => {
                    // Either `ForceNone`, or `NotForcing && era_length < T::SessionsPerEra::get()`.
                    if era_length + 1 == T::SessionsPerEra::get() {
                        IsCurrentSessionFinal::<T>::put(true);
                    } else if era_length >= T::SessionsPerEra::get() {
                        // Should only happen when we are ready to trigger an era but we have ForceNone,
                        // otherwise previous arm would short circuit.
                        Self::close_election_window();
                    }
                    return None;
                }
            }

            // new era.
            Self::new_era(session_index)
        } else {
            // Set initial era
            Self::new_era(session_index)
        }
    }

    /// Start a session potentially starting an era.
    fn start_session(start_session: SessionIndex) {
        let next_active_era = Self::active_era().map(|e| e.index + 1).unwrap_or(0);
        // This is only `Some` when current era has already progressed to the next era, while the
        // active era is one behind (i.e. in the *last session of the active era*, or *first session
        // of the new current era*, depending on how you look at it).
        if let Some(next_active_era_start_session_index) =
            Self::eras_start_session_index(next_active_era)
        {
            if next_active_era_start_session_index == start_session {
                Self::start_era(start_session);
            } else if next_active_era_start_session_index < start_session {
                // This arm should never happen, but better handle it than to stall the staking
                // pallet.
                frame_support::print("Warning: A session appears to have been skipped.");
                Self::start_era(start_session);
            }
        }
    }

    /// End a session potentially ending an era.
    fn end_session(session_index: SessionIndex) {
        if let Some(active_era) = Self::active_era() {
            if let Some(next_active_era_start_session_index) =
                Self::eras_start_session_index(active_era.index + 1)
            {
                if next_active_era_start_session_index == session_index + 1 {
                    Self::end_era(active_era, session_index);
                }
            }
        }
    }

    /// * Increment `active_era.index`,
    /// * reset `active_era.start`,
    /// * update `BondedEras` and apply slashes.
    fn start_era(start_session: SessionIndex) {
        let active_era = ActiveEra::<T>::mutate(|active_era| {
            let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
            *active_era = Some(ActiveEraInfo {
                index: new_index,
                // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
                start: None,
            });
            new_index
        });

        let bonding_duration = T::BondingDuration::get();

        BondedEras::<T>::mutate(|bonded| {
            bonded.push((active_era, start_session));

            if active_era > bonding_duration {
                let first_kept = active_era - bonding_duration;

                // prune out everything that's from before the first-kept index.
                let n_to_prune = bonded
                    .iter()
                    .take_while(|&&(era_idx, _)| era_idx < first_kept)
                    .count();

                // kill slashing metadata.
                for (pruned_era, _) in bonded.drain(..n_to_prune) {
                    slashing::clear_era_metadata::<T>(pruned_era);
                }

                if let Some(&(_, first_session)) = bonded.first() {
                    T::SessionInterface::prune_historical_up_to(first_session);
                }
            }
        });

        Self::apply_unapplied_slashes(active_era);
    }

    /// pay validator rewards based on their reward points
    fn pay_validators(era: EraIndex) {
        let remainder_mining_reward = T::NumberToCurrency::convert(
            Self::remainder_mining_reward().unwrap_or(T::TotalMiningReward::get()),
        );
        if remainder_mining_reward == Zero::zero() {
            return;
        }
        let era_payout = cmp::min(Self::era_validator_reward(), remainder_mining_reward);
        let mut total_payout = Zero::zero();
        let era_reward_points = <ErasRewardPoints<T>>::get(&era);
        let total_reward_points = era_reward_points.total;
        for validator in Self::eras_validators(era) {
            let validator_reward_points = era_reward_points
                .individual
                .get(&validator)
                .map(|points| *points)
                .unwrap_or_else(|| Zero::zero());
            if !validator_reward_points.is_zero() {
                let validator_total_reward_part =
                    Perbill::from_rational(validator_reward_points, total_reward_points);
                // This is how much validator is entitled to.
                let validator_total_payout = validator_total_reward_part * era_payout;
                total_payout += validator_total_payout;
                if let Some(imbalance) =
                    Self::make_validator_payout(&validator, validator_total_payout)
                {
                    Self::deposit_event(Event::<T>::ValidatorReward(
                        validator.clone(),
                        imbalance.peek(),
                    ));
                }
            }
        }
        RemainderMiningReward::<T>::put(
            TryInto::<u128>::try_into(remainder_mining_reward.saturating_sub(total_payout))
                .ok()
                .unwrap(),
        );
    }

    /// Validators can set reward destination or payee, so we need to handle that.
    fn make_validator_payout(
        stash: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Option<PositiveImbalanceOf<T>> {
        let dest = Self::payee(stash);
        match dest {
            RewardDestination::Controller => Self::bonded(stash)
                .and_then(|controller| Some(T::Currency::deposit_creating(&controller, amount))),
            RewardDestination::Stash => T::Currency::deposit_into_existing(stash, amount).ok(),
            RewardDestination::Staked => Self::bonded(stash)
                .and_then(|c| Self::ledger(&c).map(|l| (c, l)))
                .and_then(|(controller, mut l)| {
                    l.active += amount;
                    l.total += amount;
                    let r = T::Currency::deposit_into_existing(stash, amount).ok();
                    Self::update_ledger(&controller, &l);
                    r
                }),
            RewardDestination::Account(dest_account) => {
                Some(T::Currency::deposit_creating(&dest_account, amount))
            }
        }
    }

    /// Pay delegators based on their credit
    fn pay_delegators() -> Weight {
        let mut remainder_mining_reward = T::NumberToCurrency::convert(
            Self::remainder_mining_reward().unwrap_or(T::TotalMiningReward::get()),
        );
        let mut weight = T::DbWeight::get().reads_writes(1, 0);
        if remainder_mining_reward == Zero::zero() {
            return weight;
        }
        let prefix = Self::delegators_key_prefix(); // 1 read
        let mut last_key = Self::delegators_last_key(); // 1 read
        let mut next_key = Self::next_delegators_key(&last_key); // 1 read
        let mut counter = 0;
        let delegator_payouts_per_block = Self::delegator_payouts_per_block(); // 1 read
        let current_era = T::CreditInterface::get_current_era(); // 1 read

        weight = weight.saturating_add(T::DbWeight::get().reads_writes(5, 0));
        while next_key.starts_with(&prefix) && counter < delegator_payouts_per_block {
            let optional_delegator_data = Self::get_delegator_data(&next_key); // 1 read
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if optional_delegator_data.is_none() {
                break;
            }
            let delegator_data = optional_delegator_data.unwrap();
            let (payout, payout_weight) =
                Self::pay_delegator(&delegator_data, current_era, remainder_mining_reward);
            weight = weight.saturating_add(payout_weight);
            remainder_mining_reward = remainder_mining_reward.saturating_sub(payout);
            if remainder_mining_reward == Zero::zero() {
                break;
            }
            last_key = next_key.clone();
            next_key = Self::next_delegators_key(&last_key); // 1 read
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            counter += 1;
        }
        if counter == delegator_payouts_per_block {
            // might not be over yet
            DelegatorsLastKey::<T>::put(last_key); // persist the last key for next block
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
        }
        RemainderMiningReward::<T>::put(
            TryInto::<u128>::try_into(remainder_mining_reward)
                .ok()
                .unwrap(),
        );
        weight.saturating_add(T::DbWeight::get().reads_writes(0, 1))
    }

    fn pay_delegator(
        delegator_data: &DelegatorData<T::AccountId>,
        current_era: EraIndex,
        remainder_mining_reward: BalanceOf<T>,
    ) -> (BalanceOf<T>, Weight) {
        let earliest_unrewarded_era = delegator_data.unrewarded_since.unwrap_or(current_era);
        if earliest_unrewarded_era == current_era {
            return (BalanceOf::<T>::zero(), Weight::zero());
        }

        let delegator = &delegator_data.delegator;
        let mut payout = BalanceOf::<T>::zero();
        let mut weight = T::DbWeight::get().reads_writes(1, 0); // for im_ever_online

        let (rewards, get_reward_weight) =
            T::CreditInterface::get_reward(delegator, earliest_unrewarded_era, current_era - 1);
        weight = weight.saturating_add(get_reward_weight);
        if let Some((referee_reward, poc_reward)) = rewards {
            // update RewardData
            if Reward::<T>::contains_key(delegator) {
                // 1 read
                Reward::<T>::mutate(delegator, |data| match data {
                    // 1 write
                    Some(reward_data) => {
                        reward_data.received_referee_reward += referee_reward;
                        reward_data.referee_reward = referee_reward;
                        reward_data.received_pocr_reward += poc_reward;
                        reward_data.poc_reward = poc_reward;
                    }
                    _ => (),
                });
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            } else {
                let (total_referee_reward, get_top_referee_reward_weight) =
                    T::CreditInterface::get_top_referee_reward(delegator);
                weight = weight.saturating_add(get_top_referee_reward_weight);
                let reward_data = RewardData::<BalanceOf<T>> {
                    total_referee_reward,
                    received_referee_reward: referee_reward,
                    referee_reward: referee_reward,
                    received_pocr_reward: poc_reward,
                    poc_reward: poc_reward,
                };
                Reward::<T>::insert(delegator, reward_data); // 1 write
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
            }
            let reward = cmp::min(remainder_mining_reward, referee_reward + poc_reward);
            let imbalance = T::Currency::deposit_creating(delegator, reward); // 1 write
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
            Self::deposit_event(Event::<T>::DelegatorReward(
                (*delegator).clone(),
                imbalance.peek(),
            ));
            payout = reward;
        }
        if delegator_data.delegating {
            Delegators::<T>::mutate(delegator, |data| {
                data.unrewarded_since = Some(current_era);
            });
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
        } else {
            Delegators::<T>::remove(delegator);
            DelegatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
        }

        (payout, weight)
    }

    /// Compute payout for era.
    fn end_era(active_era: ActiveEraInfo, _session_index: SessionIndex) {
        // Note: active_era_start can be None if end era is called during genesis config.
        if let Some(_active_era_start) = active_era.start {
            Self::pay_validators(active_era.index);
        }
    }

    /// Plan a new era. Return the potential new staking set.
    fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        // Increment or set current era.
        let current_era = CurrentEra::<T>::mutate(|s| {
            *s = Some(s.map(|s| s + 1).unwrap_or(0));
            s.unwrap()
        });
        ErasStartSessionIndex::<T>::insert(&current_era, &start_session_index);

        // Clean old era information.
        if let Some(old_era) = current_era.checked_sub(Self::history_depth() + 1) {
            Self::clear_era_information(old_era);
        }

        let maybe_new_validators = Self::elect(current_era);

        if let Some(new_validators) = maybe_new_validators.clone() {
            ErasValidators::<T>::insert(current_era, new_validators);
        }

        maybe_new_validators
    }

    fn close_election_window() {
        // Close window.
        <EraElectionStatus<T>>::put(ElectionStatus::Closed);
        // Don't track final session.
        IsCurrentSessionFinal::<T>::put(false);
    }

    fn trusted_validator(account: &T::AccountId) -> bool {
        let whitelist = Self::validator_whitelist();
        whitelist.contains(account) || whitelist.is_empty()
    }

    /// elect new validators at the beginning of the era.
    ///
    /// updates the following storage items:
    /// - [`EraElectionStatus`]: with `None`.
    /// - [`ErasStakers`]: with the new staker set.
    /// - [`ErasValidatorPrefs`].
    /// - [`ErasTotalStake`]: with the new total stake.
    ///
    /// If the election has been successful, It passes the new set upwards.
    fn elect(current_era: EraIndex) -> Option<Vec<T::AccountId>> {
        let mut validators: Vec<(T::AccountId, u32, EraIndex)> = Validators::<T>::iter()
            .filter(|(validator, _)| Self::trusted_validator(&validator))
            .map(|(validator, _)| {
                // if let Some(candidate_validator) = Self::candidate_validators(&validator) {
                //     (
                //         validator.clone(),
                //         candidate_validator.delegators.len() as u32,
                //         candidate_validator.elected_era,
                //     )
                // }
                let candidate_validator = Self::candidate_validators(&validator);
                (
                    validator.clone(),
                    candidate_validator.delegators.len() as u32,
                    candidate_validator.elected_era,
                )
            })
            .collect();
        if validators.len() < Self::minimum_validator_count().max(1) as usize {
            // If we don't have enough candidate_validators, nothing to do.
            log!(
                warn,
                "💸 Chain does not have enough staking candidate_validators to operate. Era {:?}.",
                Self::current_era()
            );
            None
        } else {
            validators.sort_by(|a, b| Self::compare(&(a.1, a.2), &(b.1, b.2)));
            let truncated = validators.len() > Self::validator_count() as usize;
            validators.truncate(Self::validator_count() as usize);
            let elected_validators: Vec<T::AccountId> =
                validators.iter().map(|(v, _, _)| (*v).clone()).collect();
            for elected_validator in &elected_validators {
                <CandidateValidators<T>>::mutate(&elected_validator, |validator_data| {
                    validator_data.elected_era = current_era + 1; // makes sure it's not 0
                });
            }
            log!(
                info,
                "💸 new validator set of size {:?} has been elected for era {:?}\n 
                candidate_delegators: {:?}
                ",
                elected_validators.len(),
                current_era,
                &elected_validators
            );
            let mut total_stake: BalanceOf<T> = Zero::zero();
            for v in &elected_validators {
                let stake = Self::bonded(v)
                    .and_then(Self::ledger)
                    .map(|l| l.active)
                    .unwrap_or_default();
                // expose delegators only if not all validators elected.
                let others = if truncated {
                    let candidate = Self::candidate_validators(v);

                    candidate.delegators.into_iter().collect()
                } else {
                    Vec::new()
                };
                let exposure = Exposure {
                    total: stake,
                    own: stake,
                    others,
                };
                ErasStakers::<T>::insert(current_era, &v, exposure);
                total_stake = total_stake.saturating_add(stake);
            }
            ErasTotalStake::<T>::insert(&current_era, total_stake);
            Some(elected_validators)
        }
    }

    fn compare(a: &(u32, EraIndex), b: &(u32, EraIndex)) -> Ordering {
        match a.1.cmp(&b.1) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => b.0.cmp(&a.0),
        }
    }

    /// Remove all associated data of a stash account from the staking system.
    ///
    /// Assumes storage is upgraded before calling.
    ///
    /// This is called:
    /// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
    /// - through `reap_stash()` if the balance has fallen to zero (through slashing).
    fn kill_stash(stash: &T::AccountId, num_slashing_spans: u32) -> DispatchResult {
        let controller = <Bonded<T>>::get(stash).ok_or(Error::<T>::NotStash)?;

        slashing::clear_stash_metadata::<T>(stash, num_slashing_spans)?;

        <Bonded<T>>::remove(stash);
        <Ledger<T>>::remove(&controller);

        <Payee<T>>::remove(stash);
        <Validators<T>>::remove(stash);

        frame_system::Pallet::<T>::dec_consumers(stash);

        Ok(())
    }

    /// Clear all era information for given era.
    fn clear_era_information(era_index: EraIndex) {
        <ErasStakers<T>>::remove_prefix(era_index, None);
        <ErasValidatorPrefs<T>>::remove_prefix(era_index, None);
        <ErasRewardPoints<T>>::remove(era_index);
        <ErasTotalStake<T>>::remove(era_index);
        <ErasValidators<T>>::remove(era_index);
        <ErasStartSessionIndex<T>>::remove(era_index);
    }

    /// Apply previously-unapplied slashes on the beginning of a new era, after a delay.
    fn apply_unapplied_slashes(active_era: EraIndex) {
        let slash_defer_duration = T::SlashDeferDuration::get();
        <Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
            if let Some(ref mut earliest) = earliest {
                let keep_from = active_era.saturating_sub(slash_defer_duration);
                for era in (*earliest)..keep_from {
                    let era_slashes = <Self as Store>::UnappliedSlashes::take(&era);
                    for slash in era_slashes {
                        slashing::apply_slash::<T>(slash);
                    }
                }

                *earliest = (*earliest).max(keep_from)
            }
        })
    }

    /// Add reward points to validators using their stash account ID.
    ///
    /// Validators are keyed by stash account ID and must be in the current elected set.
    ///
    /// For each element in the iterator the given number of points in u32 is added to the
    /// validator, thus duplicates are handled.
    ///
    /// At the end of the era each the total payout will be distributed among validator
    /// relatively to their points.
    ///
    /// COMPLEXITY: Complexity is `number_of_validator_to_reward x current_elected_len`.
    /// If you need to reward lots of validator consider using `reward_by_indices`.
    pub fn reward_by_ids(validators_points: impl IntoIterator<Item = (T::AccountId, u32)>) {
        if let Some(active_era) = Self::active_era() {
            <ErasRewardPoints<T>>::mutate(active_era.index, |era_rewards| {
                let update_era_rewards = era_rewards;
                for (validator, points) in validators_points.into_iter() {
                    *update_era_rewards.individual.entry(validator).or_default() += points;
                    update_era_rewards.total += points;
                }
            });
        }
    }

    /// Ensures that at the end of the current session there will be a new era.
    fn ensure_new_era() {
        match ForceEra::<T>::get() {
            Forcing::ForceAlways | Forcing::ForceNew => (),
            _ => ForceEra::<T>::put(Forcing::ForceNew),
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    pub fn add_era_stakers(
        current_era: EraIndex,
        controller: T::AccountId,
        exposure: Exposure<T::AccountId, BalanceOf<T>>,
    ) {
        <ErasStakers<T>>::insert(&current_era, &controller, &exposure);
    }

    #[cfg(feature = "runtime-benchmarks")]
    pub fn put_election_status(status: ElectionStatus<T::BlockNumber>) {
        <EraElectionStatus<T>>::put(status);
    }

    #[cfg(feature = "runtime-benchmarks")]
    pub fn set_slash_reward_fraction(fraction: Perbill) {
        SlashRewardFraction::<T>::put(fraction);
    }

    fn _undelegate(delegator: &T::AccountId) {
        let delegator_data = Self::delegators(delegator);

        if delegator_data.delegating {
            for validator in delegator_data.delegated_validators {
                <CandidateValidators<T>>::mutate(&validator, |validator_data| {
                    validator_data.delegators.remove(delegator);
                });
                let validator_data = Self::candidate_validators(&validator);
                match validator_data.delegators.len() {
                    0 => <CandidateValidators<T>>::remove(&validator),
                    _ => (),
                }
            }

            match delegator_data.unrewarded_since {
                Some(earliest_unrewarded_era) => {
                    if earliest_unrewarded_era == CurrentEra::<T>::get().unwrap_or(0) {
                        <Delegators<T>>::remove(delegator);
                        DelegatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
                    } else {
                        <Delegators<T>>::mutate(delegator, |data| data.delegating = false);
                    }
                }
                None => {
                    // remove the delegator
                    <Delegators<T>>::remove(delegator);
                    DelegatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
                }
            }
            ActiveDelegatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
        }
    }

    fn get_delegators_prefix_hash() -> Vec<u8> {
        use frame_support::storage::generator::StorageMap;
        Delegators::<T>::prefix_hash()
    }

    fn next_delegators_key(last_key: &Vec<u8>) -> Vec<u8> {
        sp_io::storage::next_key(last_key).unwrap_or(Vec::<u8>::new())
    }

    fn get_delegator_data(next_key: &Vec<u8>) -> Option<DelegatorData<T::AccountId>> {
        frame_support::storage::unhashed::get::<DelegatorData<T::AccountId>>(next_key)
    }

    /// return values:
    /// tobe slashed credit and tobe slashed balance
    fn calc_extra_balance_credit(account: &T::AccountId) -> (u64, BalanceOf<T>) {
        use sp_runtime::traits::UniqueSaturatedFrom;
        const DPR: u128 = 1_000_000_000_000_000_000;
        const FROM_ERA: u32 = 271;

        let campaign0_reward: Vec<BalanceOf<T>> = vec![
            0u32.into(),
            UniqueSaturatedFrom::unique_saturated_from(2137 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(6026 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(11152 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(22307 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(39419 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(58389 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(82674 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(115400 * DPR / 100),
        ];
        let campaign1_reward: Vec<BalanceOf<T>> = vec![
            0u32.into(),
            UniqueSaturatedFrom::unique_saturated_from(2137 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(5642 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(10521 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(21173 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(37030 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(54444 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(75616 * DPR / 100),
            UniqueSaturatedFrom::unique_saturated_from(102575 * DPR / 100),
        ];
        let history = T::CreditInterface::get_credit_history(account);
        if history.is_empty() || (history[0].1.campaign_id != 0 && history[0].1.campaign_id != 1) {
            return (0, 0u32.into());
        }

        let rewards = {
            if history[0].1.campaign_id == 0 {
                campaign0_reward
            } else {
                campaign1_reward
            }
        };
        let cur_era = T::CreditInterface::get_current_era();
        let len = history.len();
        let mut basic_credit = 0;
        let mut slash_credit = 0;
        let mut extra_reward: BalanceOf<T> = 0u32.into();
        let mut caclulated_era = 0;
        let mut last_level: u8 = 0;
        let mut basic_level: u8 = 0;
        let mut is_first_staking = true;
        let mut last_credit = 0;

        for i in 0..len {
            let (era, credit_data) = &history[i];
            if *era <= FROM_ERA {
                // get the credit before staking delegate
                basic_credit = credit_data.credit;
                last_credit = basic_credit;
                continue;
            }
            // basic_level and basic_credit,here is const
            basic_level = T::CreditInterface::get_credit_level(basic_credit).into();

            let diff = credit_data.credit.saturating_sub(last_credit);
            // not 100,since just when staking delegate success ,slashing offline credit
            if diff >= 99 {
                slash_credit += diff;
                // before first staking_delegate,the reward is legal
                if is_first_staking {
                    is_first_staking = false;
                } else {
                    extra_reward += (rewards[last_level as usize] - rewards[basic_level as usize])
                        * ((era - caclulated_era).into());
                }
                caclulated_era = *era;
                last_level = T::CreditInterface::get_credit_level(credit_data.credit).into();
            }

            last_credit = credit_data.credit;
        }
        // not happens,just for case
        if last_level <= basic_level {
            return (0, 0u32.into());
        }

        // use cur_era-1 ,because maybe current era's reward not pay
        extra_reward += (rewards[last_level as usize] - rewards[basic_level as usize])
            * (((cur_era - 1).saturating_sub(caclulated_era)).into());
        (slash_credit, extra_reward)
    }
}

/// In this implementation `new_session(session)` must be called before `end_session(session-1)`
/// i.e. the new session must be planned before the ending of the previous session.
///
/// Once the first new_session is planned, all session must start and then end in order, though
/// some session can lag in between the newest session planned and the latest session started.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for pallet::Pallet<T> {
    fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        log::trace!(
            target: LOG_TARGET,
            "[{:#?}] planning new_session({})",
            <frame_system::Pallet<T>>::block_number(),
            new_index
        );
        Self::new_session(new_index)
    }
    fn start_session(start_index: SessionIndex) {
        log::trace!(
            target: LOG_TARGET,
            "[{:#?}] starting start_session({})",
            <frame_system::Pallet<T>>::block_number(),
            start_index
        );
        Self::start_session(start_index)
    }
    fn end_session(end_index: SessionIndex) {
        log::trace!(
            target: LOG_TARGET,
            "[{:#?}] ending end_session({})",
            <frame_system::Pallet<T>>::block_number(),
            end_index
        );
        Self::end_session(end_index)
    }
}

impl<T: Config> historical::SessionManager<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>>
    for pallet::Pallet<T>
{
    fn new_session(
        new_index: SessionIndex,
    ) -> Option<Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>> {
        <Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
            let current_era = Self::current_era()
                // Must be some as a new era has been created.
                .unwrap_or(0);

            validators
                .into_iter()
                .map(|v| {
                    let exposure = Self::eras_stakers(current_era, &v);
                    (v, exposure)
                })
                .collect()
        })
    }
    fn start_session(start_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::start_session(start_index)
    }
    fn end_session(end_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::end_session(end_index)
    }
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for pallet::Pallet<T>
where
    T: Config + pallet_authorship::Config + pallet_session::Config,
{
    fn note_author(author: T::AccountId) {
        Self::reward_by_ids(vec![(author, 20)])
    }
    fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
        if let Some(block_author) = <pallet_authorship::Pallet<T>>::author() {
            Self::reward_by_ids(vec![(block_author, 2), (author, 1)])
        }
    }
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
    fn convert(controller: T::AccountId) -> Option<T::AccountId> {
        <Pallet<T>>::ledger(&controller).map(|l| l.stash)
    }
}

/// Active exposure is the exposure of the validator set currently validating, i.e. in
/// `active_era`. It can differ from the latest planned exposure in `current_era`.
pub struct ExposureOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
    for ExposureOf<T>
{
    fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
        if let Some(active_era) = <Pallet<T>>::active_era() {
            Some(<Pallet<T>>::eras_stakers(active_era.index, &validator))
        } else {
            None
        }
    }
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Config>
    OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>, Weight>
    for pallet::Pallet<T>
where
    T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
    T: pallet_session::historical::Config<
        FullIdentification = Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        FullIdentificationOf = ExposureOf<T>,
    >,
    T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Config>::AccountId>,
    T::SessionManager: pallet_session::SessionManager<<T as frame_system::Config>::AccountId>,
    T::ValidatorIdOf: Convert<
        <T as frame_system::Config>::AccountId,
        Option<<T as frame_system::Config>::AccountId>,
    >,
{
    fn on_offence(
        offenders: &[OffenceDetails<
            T::AccountId,
            pallet_session::historical::IdentificationTuple<T>,
        >],
        slash_fraction: &[Perbill],
        slash_session: SessionIndex,
        _disable_strategy: DisableStrategy,
    ) -> Weight {
        if !Self::era_election_status().is_closed() {
            return 0;
        }

        let reward_proportion = SlashRewardFraction::<T>::get();
        let mut consumed_weight: Weight = 0;
        let mut add_db_reads_writes = |reads, writes| {
            consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
        };

        let active_era = {
            let active_era = Self::active_era();
            add_db_reads_writes(1, 0);
            if active_era.is_none() {
                // this offence need not be re-submitted.
                return consumed_weight;
            }
            active_era
                .expect("value checked not to be `None`; qed")
                .index
        };
        let active_era_start_session_index = Self::eras_start_session_index(active_era)
            .unwrap_or_else(|| {
                frame_support::print("Error: start_session_index must be set for current_era");
                0
            });
        add_db_reads_writes(1, 0);

        let window_start = active_era.saturating_sub(T::BondingDuration::get());

        // fast path for active-era report - most likely.
        // `slash_session` cannot be in a future active era. It must be in `active_era` or before.
        let slash_era = if slash_session >= active_era_start_session_index {
            active_era
        } else {
            let eras = BondedEras::<T>::get();
            add_db_reads_writes(1, 0);

            // reverse because it's more likely to find reports from recent eras.
            match eras
                .iter()
                .rev()
                .find(|&&(_, ref sesh)| sesh <= &slash_session)
            {
                Some(&(ref slash_era, _)) => *slash_era,
                // before bonding period. defensive - should be filtered out.
                None => return consumed_weight,
            }
        };

        <Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
            if earliest.is_none() {
                *earliest = Some(active_era)
            }
        });
        add_db_reads_writes(1, 1);

        let slash_defer_duration = T::SlashDeferDuration::get();

        let invulnerables = Self::invulnerables();
        add_db_reads_writes(1, 0);

        for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
            let (stash, exposure) = &details.offender;

            // Skip if the validator is invulnerable.
            if invulnerables.contains(stash) {
                continue;
            }

            let unapplied = slashing::compute_slash::<T>(slashing::SlashParams {
                stash,
                slash: *slash_fraction,
                exposure,
                slash_era,
                window_start,
                now: active_era,
                reward_proportion,
            });

            if let Some(mut unapplied) = unapplied {
                let delegators_len = unapplied.others.len() as u64;
                let reporters_len = details.reporters.len() as u64;

                {
                    let upper_bound = 1 /* Validator/NominatorSlashInEra */ + 2 /* fetch_spans */;
                    let rw = upper_bound + delegators_len * upper_bound;
                    add_db_reads_writes(rw, rw);
                }
                unapplied.reporters = details.reporters.clone();
                if slash_defer_duration == 0 {
                    // apply right away.
                    slashing::apply_slash::<T>(unapplied);
                    {
                        let slash_cost = (6, 5);
                        let reward_cost = (2, 2);
                        add_db_reads_writes(
                            (1 + delegators_len) * slash_cost.0 + reward_cost.0 * reporters_len,
                            (1 + delegators_len) * slash_cost.1 + reward_cost.1 * reporters_len,
                        );
                    }
                } else {
                    // defer to end of some `slash_defer_duration` from now.
                    <Self as Store>::UnappliedSlashes::mutate(active_era, move |for_later| {
                        for_later.push(unapplied)
                    });
                    add_db_reads_writes(1, 1);
                }
            } else {
                add_db_reads_writes(4 /* fetch_spans */, 5 /* kick_out_if_recent */)
            }
        }

        consumed_weight
    }
}

/// Filter historical offences out and only allow those from the bonding period.
pub struct FilterHistoricalOffences<T, R> {
    _inner: sp_std::marker::PhantomData<(T, R)>,
}

impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O>
    for FilterHistoricalOffences<Pallet<T>, R>
where
    T: Config,
    R: ReportOffence<Reporter, Offender, O>,
    O: Offence<Offender>,
{
    fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), OffenceError> {
        // disallow any slashing from before the current bonding period.
        let offence_session = offence.session_index();
        let bonded_eras = BondedEras::<T>::get();

        if bonded_eras
            .first()
            .filter(|(_, start)| offence_session >= *start)
            .is_some()
        {
            R::report_offence(reporters, offence)
        } else {
            <Pallet<T>>::deposit_event(Event::<T>::OldSlashingReportDiscarded(offence_session));
            Ok(())
        }
    }

    fn is_known_offence(offenders: &[Offender], time_slot: &O::TimeSlot) -> bool {
        R::is_known_offence(offenders, time_slot)
    }
}

/// Check that list is sorted and has no duplicates.
fn is_sorted_and_unique(list: &[u32]) -> bool {
    list.windows(2).all(|w| w[0] < w[1])
}
