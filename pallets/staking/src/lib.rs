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

// # Staking Module
//
// The Staking module is used to manage funds at stake by network maintainers.
//
// - [`staking::Config`](./trait.Config.html)
// - [`Call`](./enum.Call.html)
// - [`Module`](./struct.Module.html)
//
// ## Overview
//
// The Staking module is the means by which a set of network maintainers (known as _authorities_ in
// some contexts and _validators_ in others) are chosen based upon those who voluntarily place
// funds under deposit. Under deposit, those funds are rewarded under normal operation but are held
// at pain of _slash_ (expropriation) should the staked maintainer be found not to be discharging
// its duties properly.
//
// ### Terminology
// <!-- Original author of paragraph: @gavofyork -->
//
// - Staking: The process of locking up funds for some time, placing them at risk of slashing
//   (loss) in order to become a rewarded maintainer of the network.
// - Validating: The process of running a node to actively maintain the network, either by
//   producing blocks or guaranteeing finality of the chain.
// - Nominating: The process of placing staked funds behind one or more validators in order to
//   share in any reward, and punishment, they take.
// - Stash account: The account holding an owner's funds used for staking.
// - Controller account: The account that controls an owner's funds for staking.
// - Era: A (whole) number of sessions, which is the period that the validator set (and each
//   validator's active nominator set) is recalculated and where rewards are paid out.
// - Slash: The punishment of a staker by reducing its funds.
//
// ### Goals
// <!-- Original author of paragraph: @gavofyork -->
//
// The staking system in Substrate NPoS is designed to make the following possible:
//
// - Stake funds that are controlled by a cold wallet.
// - Withdraw some, or deposit more, funds without interrupting the role of an entity.
// - Switch between roles (nominator, validator, idle) with minimal overhead.
//
// ### Scenarios
//
// #### Staking
//
// Almost any interaction with the Staking module requires a process of _**bonding**_ (also known
// as being a _staker_). To become *bonded*, a fund-holding account known as the _stash account_,
// which holds some or all of the funds that become frozen in place as part of the staking process,
// is paired with an active **controller** account, which issues instructions on how they shall be
// used.
//
// An account pair can become bonded using the [`bond`](./enum.Call.html#variant.bond) call.
//
// Stash accounts can change their associated controller using the
// [`set_controller`](./enum.Call.html#variant.set_controller) call.
//
// There are three possible roles that any staked account pair can be in: `Validator`, `Nominator`
// and `Idle` (defined in [`StakerStatus`](./enum.StakerStatus.html)). There are three
// corresponding instructions to change between roles, namely:
// [`validate`](./enum.Call.html#variant.validate),
// [`nominate`](./enum.Call.html#variant.nominate), and [`chill`](./enum.Call.html#variant.chill).
//
// #### Validating
//
// A **validator** takes the role of either validating blocks or ensuring their finality,
// maintaining the veracity of the network. A validator should avoid both any sort of malicious
// misbehavior and going offline. Bonded accounts that state interest in being a validator do NOT
// get immediately chosen as a validator. Instead, they are declared as a _candidate_ and they
// _might_ get elected at the _next era_ as a validator. The result of the election is determined
// by nominators and their votes.
//
// An account can become a validator candidate via the
// [`validate`](./enum.Call.html#variant.validate) call.
//
// #### Nomination
//
// A **nominator** does not take any _direct_ role in maintaining the network, instead, it votes on
// a set of validators  to be elected. Once interest in nomination is stated by an account, it
// takes effect at the next election round. The funds in the nominator's stash account indicate the
// _weight_ of its vote. Both the rewards and any punishment that a validator earns are shared
// between the validator and its nominators. This rule incentivizes the nominators to NOT vote for
// the misbehaving/offline validators as much as possible, simply because the nominators will also
// lose funds if they vote poorly.
//
// An account can become a nominator via the [`nominate`](enum.Call.html#variant.nominate) call.
//
// #### Rewards and Slash
//
// The **reward and slashing** procedure is the core of the Staking module, attempting to _embrace
// valid behavior_ while _punishing any misbehavior or lack of availability_.
//
// Rewards must be claimed for each era before it gets too old by `$HISTORY_DEPTH` using the
// `payout_stakers` call. Any account can call `payout_stakers`, which pays the reward to the
// validator as well as its nominators. Only the [`Config::MaxNominatorRewardedPerValidator`]
// biggest stakers can claim their reward. This is to limit the i/o cost to mutate storage for each
// nominator's account.
//
// Slashing can occur at any point in time, once misbehavior is reported. Once slashing is
// determined, a value is deducted from the balance of the validator and all the nominators who
// voted for this validator (values are deducted from the _stash_ account of the slashed entity).
//
// Slashing logic is further described in the documentation of the `slashing` module.
//
// Similar to slashing, rewards are also shared among a validator and its associated nominators.
// Yet, the reward funds are not always transferred to the stash account and can be configured. See
// [Reward Calculation](#reward-calculation) for more details.
//
// #### Chilling
//
// Finally, any of the roles above can choose to step back temporarily and just chill for a while.
// This means that if they are a nominator, they will not be considered as voters anymore and if
// they are validators, they will no longer be a candidate for the next election.
//
// An account can step back via the [`chill`](enum.Call.html#variant.chill) call.
//
// ### Session managing
//
// The module implement the trait `SessionManager`. Which is the only API to query new validator
// set and allowing these validator set to be rewarded once their era is ended.
//
// ## Interface
//
// ### Dispatchable Functions
//
// The dispatchable functions of the Staking module enable the steps needed for entities to accept
// and change their role, alongside some helper functions to get/set the metadata of the module.
//
// ### Public Functions
//
// The Staking module contains many public storage items and (im)mutable functions.
//
// ## Usage
//
// ### Example: Rewarding a validator by id.
//
// ```
// use frame_support::{decl_module, dispatch};
// use frame_system::ensure_signed;
// use pallet_staking::{self as staking};
//
// pub trait Config: staking::Config {}
//
// decl_module! {
//     pub struct Module<T: Config> for enum Call where origin: T::Origin {
//         /// Reward a validator.
//         #[weight = 0]
//         pub fn reward_myself(origin) -> dispatch::DispatchResult {
//             let reported = ensure_signed(origin)?;
//             <staking::Module<T>>::reward_by_ids(vec![(reported, 10)]);
//             Ok(())
//         }
//     }
// }
// # fn main() { }
// ```
//
// ## Implementation Details
//
// ### Era payout
//
// The era payout is computed using yearly inflation curve defined at
// [`T::RewardCurve`](./trait.Config.html#associatedtype.RewardCurve) as such:
//
// ```nocompile
// staker_payout = yearly_inflation(npos_token_staked / total_tokens) * total_tokens / era_per_year
// ```
// This payout is used to reward stakers as defined in next section
//
// ```nocompile
// remaining_payout = max_yearly_inflation * total_tokens / era_per_year - staker_payout
// ```
// The remaining reward is send to the configurable end-point
// [`T::RewardRemainder`](./trait.Config.html#associatedtype.RewardRemainder).
//
// ### Reward Calculation
//
// Validators and nominators are rewarded at the end of each era. The total reward of an era is
// calculated using the era duration and the staking rate (the total amount of tokens staked by
// nominators and validators, divided by the total token supply). It aims to incentivize toward a
// defined staking rate. The full specification can be found
// [here](https://research.web3.foundation/en/latest/polkadot/Token%20Economics.html#inflation-model).
//
// Total reward is split among validators and their nominators depending on the number of points
// they received during the era. Points are added to a validator using
// [`reward_by_ids`](./enum.Call.html#variant.reward_by_ids) or
// [`reward_by_indices`](./enum.Call.html#variant.reward_by_indices).
//
// [`Module`](./struct.Module.html) implements
// [`pallet_authorship::EventHandler`](../pallet_authorship/trait.EventHandler.html) to add reward
// points to block producer and block producer of referenced uncles.
//
// The validator and its nominator split their reward as following:
//
// The validator can declare an amount, named
// [`commission`](./struct.ValidatorPrefs.html#structfield.commission), that does not get shared
// with the nominators at each reward payout through its
// [`ValidatorPrefs`](./struct.ValidatorPrefs.html). This value gets deducted from the total reward
// that is paid to the validator and its nominators. The remaining portion is split among the
// validator and all of the nominators that nominated the validator, proportional to the value
// staked behind this validator (_i.e._ dividing the
// [`own`](./struct.Exposure.html#structfield.own) or
// [`others`](./struct.Exposure.html#structfield.others) by
// [`total`](./struct.Exposure.html#structfield.total) in [`Exposure`](./struct.Exposure.html)).
//
// All entities who receive a reward have the option to choose their reward destination through the
// [`Payee`](./struct.Payee.html) storage item (see
// [`set_payee`](enum.Call.html#variant.set_payee)), to be one of the following:
//
// - Controller account, (obviously) not increasing the staked value.
// - Stash account, not increasing the staked value.
// - Stash account, also increasing the staked value.
//
// ### Additional Fund Management Operations
//
// Any funds already placed into stash can be the target of the following operations:
//
// The controller account can free a portion (or all) of the funds using the
// [`unbond`](enum.Call.html#variant.unbond) call. Note that the funds are not immediately
// accessible. Instead, a duration denoted by
// [`BondingDuration`](./trait.Config.html#associatedtype.BondingDuration) (in number of eras) must
// pass until the funds can actually be removed. Once the `BondingDuration` is over, the
// [`withdraw_unbonded`](./enum.Call.html#variant.withdraw_unbonded) call can be used to actually
// withdraw the funds.
//
// Note that there is a limitation to the number of fund-chunks that can be scheduled to be
// unlocked in the future via [`unbond`](enum.Call.html#variant.unbond). In case this maximum
// (`MAX_UNLOCKING_CHUNKS`) is reached, the bonded account _must_ first wait until a successful
// call to `withdraw_unbonded` to remove some of the chunks.
//
// ### Election Algorithm
//
// The current election algorithm is implemented based on Phragmén. The reference implementation
// can be found [here](https://github.com/w3f/consensus/tree/master/NPoS).
//
// The election algorithm, aside from electing the validators with the most stake value and votes,
// tries to divide the nominator votes among candidates in an equal manner. To further assure this,
// an optional post-processing can be applied that iteratively normalizes the nominator staked
// values until the total difference among votes of a particular nominator are less than a
// threshold.
//
// ## GenesisConfig
//
// The Staking module depends on the [`GenesisConfig`](./struct.GenesisConfig.html). The
// `GenesisConfig` is optional and allow to set some initial stakers.
//
// ## Related Modules
//
// - [Balances](../pallet_balances/index.html): Used to manage values at stake.
// - [Session](../pallet_session/index.html): Used to manage sessions. Also, a list of new
//   validators is stored in the Session module's `Validators` at the end of each era.

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod testing_utils;
#[cfg(test)]
mod tests;

pub mod slashing;
pub mod weights;

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
    ensure,
    storage::IterableStorageMap,
    traits::{
        Currency, CurrencyToVote, EnsureOrigin, EstimateNextNewSession, Get, Imbalance, IsSubType,
        LockIdentifier, LockableCurrency, OnUnbalanced, UnixTime, WithdrawReasons,
    },
    weights::{
        constants::{WEIGHT_PER_MICROS, WEIGHT_PER_NANOS},
        Weight,
    },
};
use frame_system::{self as system, ensure_root, ensure_signed, offchain::SendTransactionTypes};
use pallet_credit::CreditInterface;
use pallet_deeper_node::NodeInterface;
use pallet_session::historical;
use sp_npos_elections::{
    generate_solution_type, is_score_better, CompactSolution, ElectionScore, ExtendedBalance,
    SupportMap, VoteWeight,
};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedSub, Convert, Dispatchable, SaturatedConversion, Saturating,
        StaticLookup, Zero,
    },
    transaction_validity::TransactionPriority,
    PerU16, Perbill, Percent, RuntimeDebug,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_staking::{
    offence::{Offence, OffenceDetails, OffenceError, OnOffenceHandler, ReportOffence},
    SessionIndex,
};
use sp_std::{
    collections::btree_map::BTreeMap,
    convert::{From, TryInto},
    mem::size_of,
    prelude::*,
};
pub use weights::WeightInfo;

const STAKING_ID: LockIdentifier = *b"staking ";
pub const MAX_UNLOCKING_CHUNKS: usize = 32;
pub const MAX_NOMINATIONS: usize = <CompactAssignments as CompactSolution>::LIMIT;
use log::error;

pub(crate) const LOG_TARGET: &'static str = "staking";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		frame_support::debug::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

/// Data type used to index nominators in the compact type
pub type NominatorIndex = u32;

/// Data type used to index validators in the compact type.
pub type ValidatorIndex = u16;

// Ensure the size of both ValidatorIndex and NominatorIndex. They both need to be well below usize.
static_assertions::const_assert!(size_of::<ValidatorIndex>() <= size_of::<usize>());
static_assertions::const_assert!(size_of::<NominatorIndex>() <= size_of::<usize>());
static_assertions::const_assert!(size_of::<ValidatorIndex>() <= size_of::<u32>());
static_assertions::const_assert!(size_of::<NominatorIndex>() <= size_of::<u32>());

/// Maximum number of stakers that can be stored in a snapshot.
pub(crate) const MAX_VALIDATORS: usize = ValidatorIndex::max_value() as usize;
pub(crate) const MAX_NOMINATORS: usize = NominatorIndex::max_value() as usize;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Counter for the number of "reward" points earned by a given validator.
pub type RewardPoint = u32;

// Note: Maximum nomination limit is set here -- 16.
generate_solution_type!(
    #[compact]
    pub struct CompactAssignments::<NominatorIndex, ValidatorIndex, OffchainAccuracy>(16)
);

/// Accuracy used for on-chain election.
pub type ChainAccuracy = Perbill;

/// Accuracy used for off-chain election. This better be small.
pub type OffchainAccuracy = PerU16;

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
#[derive(Encode, Decode, RuntimeDebug)]
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
/// This points will be used to reward validators and their respective nominators.
#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug)]
pub struct EraRewardPoints<AccountId: Ord> {
    /// Total number of points. Equals the sum of reward points for each validator.
    total: RewardPoint,
    /// The reward points earned by a given validator.
    individual: BTreeMap<AccountId, RewardPoint>,
}

/// Indicates the initial status of the staker.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
    /// Chilling.
    Idle,
    /// Declared desire in validating or already participating in it.
    Validator,
    /// Nominating for a group of other stakers.
    Nominator(Vec<AccountId>),
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
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

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct RewardData<Balance: HasCompact> {
    pub total_referee_reward: Balance,
    pub received_referee_reward: Balance,
    pub daily_referee_reward: Balance,
    pub received_pocr_reward: Balance,
    pub daily_poc_reward: Balance,
}

/// Preference of what happens regarding validation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorPrefs {
    /// Reward that validator takes up-front; only the rest is split between themselves and
    /// nominators.
    #[codec(compact)]
    pub commission: Perbill,
    /// Whether or not this validator is accepting more nominations. If `true`, then no nominator
    /// who is not already nominating this validator may nominate them. By default, validators
    /// are accepting nominations.
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
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UnlockChunk<Balance: HasCompact> {
    /// Amount of funds to be unlocked.
    #[codec(compact)]
    value: Balance,
    /// Era number at which point it'll be unlocked.
    #[codec(compact)]
    era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
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

/// A record of the nominations made by a specific account.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Nominations<AccountId> {
    /// The targets of nomination.
    pub targets: Vec<AccountId>,
    /// The era the nominations were submitted.
    ///
    /// Except for initial nominations which are considered submitted at era 0.
    pub submitted_in: EraIndex,
    /// Whether the nominations have been suppressed. This can happen due to slashing of the
    /// validators, or other events that might invalidate the nomination.
    ///
    /// NOTE: this for future proofing and is thus far not used.
    pub suppressed: bool,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
    /// The stash account of the nominator in question.
    pub who: AccountId,
    /// Amount of funds exposed.
    #[codec(compact)]
    pub value: Balance,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, Balance: HasCompact> {
    /// The total balance backing this validator.
    #[codec(compact)]
    pub total: Balance,
    /// The validator's own stash that is exposed.
    #[codec(compact)]
    pub own: Balance,
    /// The portions of nominators stashes that are exposed.
    pub others: Vec<IndividualExposure<AccountId, Balance>>,
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct UnappliedSlash<AccountId, Balance: HasCompact> {
    /// The stash ID of the offending validator.
    validator: AccountId,
    /// The validator's own slash.
    own: Balance,
    /// All other slashed stakers and amounts.
    others: Vec<(AccountId, Balance)>,
    /// Reporters of the offence; bounty payout recipients.
    reporters: Vec<AccountId>,
    /// The amount of payout.
    payout: Balance,
}

/// Indicate how an election round was computed.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
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
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum ElectionStatus<BlockNumber> {
    /// Nothing has and will happen for now. submission window is not open.
    Closed,
    /// The submission window has been open since the contained block number.
    Open(BlockNumber),
}

/// Some indications about the size of the election. This must be submitted with the solution.
///
/// Note that these values must reflect the __total__ number, not only those that are present in the
/// solution. In short, these should be the same size as the size of the values dumped in
/// `SnapshotValidators` and `SnapshotNominators`.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, Default)]
pub struct ElectionSize {
    /// Number of validators in the snapshot of the current election round.
    #[codec(compact)]
    pub validators: ValidatorIndex,
    /// Number of nominators in the snapshot of the current election round.
    #[codec(compact)]
    pub nominators: NominatorIndex,
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
        <pallet_session::Module<T>>::disable(validator)
    }

    fn validators() -> Vec<<T as frame_system::Config>::AccountId> {
        <pallet_session::Module<T>>::validators()
    }

    fn prune_historical_up_to(up_to: SessionIndex) {
        <pallet_session::historical::Module<T>>::prune_up_to(up_to);
    }
}

pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
    /// The staking balance.
    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// Credit CreditInterface
    type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;

    /// Deepernode  NodeInterface
    type NodeInterface: NodeInterface<Self::AccountId>;

    /// max validators can be selected to delegate
    type MaxValidatorsCanSelected: Get<usize>;

    /// Time used for computing era duration.
    ///
    /// It is guaranteed to start being called from the first `on_finalize`. Thus value at genesis
    /// is not used.
    type UnixTime: UnixTime;

    /// Convert a balance into a number used for election calculation. This must fit into a `u64`
    /// but is allowed to be sensibly lossy. The `u64` is used to communicate with the
    /// [`sp_npos_elections`] crate which accepts u64 numbers and does operations in 128.
    /// Consequently, the backward convert is used convert the u128s from sp-elections back to a
    /// [`BalanceOf`].
    type CurrencyToVote: CurrencyToVote<BalanceOf<Self>>;

    type CurrencyToNumber: Convert<BalanceOf<Self>, u64> + Convert<u128, BalanceOf<Self>>;

    /// Tokens have been minted and are unused for validator-reward.
    /// See [Era payout](./index.html#era-payout).
    type RewardRemainder: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// Handler for the unbalanced reduction when slashing a staker.
    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Handler for the unbalanced increment when rewarding a staker.
    type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

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

    /// Something that can estimate the next session change, accurately or as a best effort guess.
    type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

    /// The number of blocks before the end of the era from which election submissions are allowed.
    ///
    /// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
    /// be used.
    ///
    /// This is bounded by being within the last session. Hence, setting it to a value more than the
    /// length of a session will be pointless.
    type ElectionLookahead: Get<Self::BlockNumber>;

    /// The overarching call type.
    type Call: Dispatchable + From<Call<Self>> + IsSubType<Call<Self>> + Clone;

    /// Maximum number of balancing iterations to run in the offchain submission.
    ///
    /// If set to 0, balance_solution will not be executed at all.
    type MaxIterations: Get<u32>;

    /// The threshold of improvement that should be provided for a new solution to be accepted.
    type MinSolutionScoreBump: Get<Perbill>;

    /// The maximum number of nominators rewarded for each validator.
    ///
    /// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
    /// their reward. This used to limit the i/o cost for the nominator payout.
    type MaxNominatorRewardedPerValidator: Get<u32>;

    /// A configuration for base priority of unsigned transactions.
    ///
    /// This is exposed so that it can be tuned for particular runtime, when
    /// multiple pallets send unsigned transactions.
    type UnsignedPriority: Get<TransactionPriority>;

    /// Maximum weight that the unsigned transaction can have.
    ///
    /// Chose this value with care. On one hand, it should be as high as possible, so the solution
    /// can contain as many nominators/validators as possible. On the other hand, it should be small
    /// enough to fit in the block.
    type OffchainSolutionWeightLimit: Get<Weight>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;

    /// credit to token factor
    type CreditToTokenFactor: Get<u128>;

    type RewardAdjustFactor: Get<u128>;

    type RewardPerBlock: Get<u128>;

    type RewardAdjustPeriod: Get<u32>;

    type BlocksPerEra: Get<u128>;

    type RemainderMiningReward: Get<u128>;
}

/// info of credit delegating
#[derive(Decode, Encode, Default)]
pub struct CreditDelegateInfo<AccountId> {
    delegator: AccountId,
    score: u64,
    validators: Vec<AccountId>,
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
    V1_0_0Ancient,
    V2_0_0,
    V3_0_0,
    V4_0_0,
    V5_0_0,
}

impl Default for Releases {
    fn default() -> Self {
        Releases::V5_0_0
    }
}

decl_storage! {
    trait Store for Module<T: Config> as Staking {
        /// Number of eras to keep in history.
        ///
        /// Information is kept for eras in `[current_era - history_depth; current_era]`.
        ///
        /// Must be more than the number of eras delayed by session otherwise. I.e. active era must
        /// always be in history. I.e. `active_era > current_era - history_depth` must be
        /// guaranteed.
        HistoryDepth get(fn history_depth) config(): u32 = 84;

        /// The ideal number of staking participants.
        pub ValidatorCount get(fn validator_count) config(): u32;

        /// Minimum number of staking participants before emergency conditions are imposed.
        pub MinimumValidatorCount get(fn minimum_validator_count) config(): u32;

        /// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
        /// easy to initialize and the performance hit is minimal (we expect no more than four
        /// invulnerables) and restricted to testnets.
        pub Invulnerables get(fn invulnerables) config(): Vec<T::AccountId>;

        /// ValidtorWhiteList
        pub ValidatorWhiteList get(fn validator_whiltelist) : Vec<T::AccountId>;

        /// Map from all locked "stash" accounts to the controller account.
        pub Bonded get(fn bonded): map hasher(twox_64_concat) T::AccountId => Option<T::AccountId>;

        /// Map from all (unlocked) "controller" accounts to the info regarding the staking.
        pub Ledger get(fn ledger):
            map hasher(blake2_128_concat) T::AccountId
            => Option<StakingLedger<T::AccountId, BalanceOf<T>>>;

        /// Where the reward payment should be made. Keyed by stash.
        pub Payee get(fn payee): map hasher(twox_64_concat) T::AccountId => RewardDestination<T::AccountId>;

        /// The map from (wannabe) validator stash key to the preferences of that validator.
        pub Validators get(fn validators):
            map hasher(twox_64_concat) T::AccountId => ValidatorPrefs;

        /// The current era index.
        ///
        /// This is the latest planned era, depending on how the Session pallet queues the validator
        /// set, it might be active or not.
        pub CurrentEra get(fn current_era): Option<EraIndex>;

        /// The active era information, it holds index and start.
        ///
        /// The active era is the era being currently rewarded. Validator set of this era must be
        /// equal to [`SessionInterface::validators`].
        pub ActiveEra get(fn active_era): Option<ActiveEraInfo>;

        /// The session index at which the era start for the last `HISTORY_DEPTH` eras.
        ///
        /// Note: This tracks the starting session (i.e. session index when era start being active)
        /// for the eras in `[CurrentEra - HISTORY_DEPTH, CurrentEra]`.
        pub ErasStartSessionIndex get(fn eras_start_session_index):
            map hasher(twox_64_concat) EraIndex => Option<SessionIndex>;

        /// Exposure of validator at era.
        ///
        /// This is keyed first by the era index to allow bulk deletion and then the stash account.
        ///
        /// Is it removed after `HISTORY_DEPTH` eras.
        /// If stakers hasn't been set or has been removed then empty exposure is returned.
        pub ErasStakers get(fn eras_stakers):
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Exposure<T::AccountId, BalanceOf<T>>;

        /// Clipped Exposure of validator at era.
        ///
        /// This is similar to [`ErasStakers`] but number of nominators exposed is reduced to the
        /// `T::MaxNominatorRewardedPerValidator` biggest stakers.
        /// (Note: the field `total` and `own` of the exposure remains unchanged).
        /// This is used to limit the i/o cost for the nominator payout.
        ///
        /// This is keyed fist by the era index to allow bulk deletion and then the stash account.
        ///
        /// Is it removed after `HISTORY_DEPTH` eras.
        /// If stakers hasn't been set or has been removed then empty exposure is returned.
        pub ErasStakersClipped get(fn eras_stakers_clipped):
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Exposure<T::AccountId, BalanceOf<T>>;

        /// Similar to `ErasStakers`, this holds the preferences of validators.
        ///
        /// This is keyed first by the era index to allow bulk deletion and then the stash account.
        ///
        /// Is it removed after `HISTORY_DEPTH` eras.
        // If prefs hasn't been set or has been removed then 0 commission is returned.
        pub ErasValidatorPrefs get(fn eras_validator_prefs):
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => ValidatorPrefs;

        /// The total validator era payout for the last `HISTORY_DEPTH` eras.
        ///
        /// Eras that haven't finished yet or has been removed doesn't have reward.
        pub ErasValidatorReward get(fn eras_validator_reward):
            map hasher(twox_64_concat) EraIndex => Option<BalanceOf<T>>;

        /// Rewards for the last `HISTORY_DEPTH` eras.
        /// If reward hasn't been set or has been removed then 0 reward is returned.
        pub ErasRewardPoints get(fn eras_reward_points):
            map hasher(twox_64_concat) EraIndex => EraRewardPoints<T::AccountId>;

        /// The total amount staked for the last `HISTORY_DEPTH` eras.
        /// If total hasn't been set or has been removed then 0 stake is returned.
        pub ErasTotalStake get(fn eras_total_stake):
            map hasher(twox_64_concat) EraIndex => BalanceOf<T>;

        /// Mode of era forcing.
        pub ForceEra get(fn force_era) config(): Forcing;

        /// The percentage of the slash that is distributed to reporters.
        ///
        /// The rest of the slashed value is handled by the `Slash`.
        pub SlashRewardFraction get(fn slash_reward_fraction) config(): Perbill;

        /// The amount of currency given to reporters of a slash event which was
        /// canceled by extraordinary circumstances (e.g. governance).
        pub CanceledSlashPayout get(fn canceled_payout) config(): BalanceOf<T>;

        /// All unapplied slashes that are queued for later.
        pub UnappliedSlashes:
            map hasher(twox_64_concat) EraIndex => Vec<UnappliedSlash<T::AccountId, BalanceOf<T>>>;

        /// A mapping from still-bonded eras to the first session index of that era.
        ///
        /// Must contains information for eras for the range:
        /// `[active_era - bounding_duration; active_era]`
        BondedEras: Vec<(EraIndex, SessionIndex)>;

        /// All slashing events on validators, mapped by era to the highest slash proportion
        /// and slash value of the era.
        ValidatorSlashInEra:
            double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
            => Option<(Perbill, BalanceOf<T>)>;

        /// Slashing spans for stash accounts.
        SlashingSpans get(fn slashing_spans): map hasher(twox_64_concat) T::AccountId => Option<slashing::SlashingSpans>;

        /// Records information about the maximum slash of a stash within a slashing span,
        /// as well as how much reward has been paid out.
        SpanSlash:
            map hasher(twox_64_concat) (T::AccountId, slashing::SpanIndex)
            => slashing::SpanRecord<BalanceOf<T>>;

        /// The earliest era for which we have a pending, unapplied slash.
        EarliestUnappliedSlash: Option<EraIndex>;

        /// Snapshot of validators at the beginning of the current election window. This should only
        /// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
        pub SnapshotValidators get(fn snapshot_validators): Option<Vec<T::AccountId>>;

        /// The next validator set. At the end of an era, if this is available (potentially from the
        /// result of an offchain worker), it is immediately used. Otherwise, the on-chain election
        /// is executed.
        pub QueuedElected get(fn queued_elected): Option<ElectionResult<T::AccountId, BalanceOf<T>>>;

        /// The score of the current [`QueuedElected`].
        pub QueuedScore get(fn queued_score): Option<ElectionScore>;

        /// Flag to control the execution of the offchain election. When `Open(_)`, we accept
        /// solutions to be submitted.
        pub EraElectionStatus get(fn era_election_status): ElectionStatus<T::BlockNumber>;

        /// True if the current **planned** session is final. Note that this does not take era
        /// forcing into account.
        pub IsCurrentSessionFinal get(fn is_current_session_final): bool = false;

        pub BlockReward get(fn block_reward): Option<u128>;

        pub RemainderMiningReward get(fn remainder_mining_reward): Option<u128>;

        /// (delegator) -> CreditDelegateInfo{}
        DelegatedToValidators get(fn delegated_to_validators): map hasher(blake2_128_concat) T::AccountId => CreditDelegateInfo<T::AccountId>;
        /// (delegator, validator) -> bool
        HasDelegatedToValidator get(fn has_delegated_to_validator): double_map hasher(blake2_128_concat) T::AccountId
        , hasher(blake2_128_concat) T::AccountId => Option<bool>;

        /// Candidate delegators info
        /// (candidateValidator) -> Vec<(delegator, score)>
        CandidateDelegators get(fn candidate_delegators): map hasher(blake2_128_concat) T::AccountId => Vec<(T::AccountId, u64)>;

        /// (EraIndex, validatorId) -> Vec<(delegator, score, hasSlashed)>
        SelectedDelegators get(fn selected_delegators): double_map hasher(blake2_128_concat) EraIndex,
        hasher(blake2_128_concat) T::AccountId => Vec<(T::AccountId, u64, bool)>;

        /// (EraIndex, delegator) -> bool
        Delegators get(fn get_delegators): double_map hasher(blake2_128_concat) EraIndex,
            hasher(blake2_128_concat) T::AccountId => Option<bool>;

        /// current era validators
        /// new era  Return the potential new staking set.
        pub CurrentEraValidators get(fn current_era_validators): Option<Vec<T::AccountId>>;

        ///  candidate validator list: CandidateValidators == Validators
        pub CandidateValidators get(fn candidate_validators): Option<Vec<T::AccountId>>;

        /// reward of delegator
        pub Reward get(fn reward): map hasher(blake2_128_concat) T::AccountId => Option<RewardData<BalanceOf<T>>>;

        /// True if network has been upgraded to this version.
        /// Storage version of the pallet.
        ///
        /// This is set to v5.0.0 for new networks.
        StorageVersion build(|_: &GenesisConfig<T>| Releases::V5_0_0): Releases;
    }
    add_extra_genesis {
        config(stakers):
            Vec<(T::AccountId, T::AccountId, BalanceOf<T>, StakerStatus<T::AccountId>)>;
        build(|config: &GenesisConfig<T>| {
            for &(ref stash, ref controller, balance, ref status) in &config.stakers {
                assert!(
                    T::Currency::free_balance(&stash) >= balance,
                    "Stash does not have enough balance to bond."
                );
                let _ = <Module<T>>::bond(
                    T::Origin::from(Some(stash.clone()).into()),
                    T::Lookup::unlookup(controller.clone()),
                    balance,
                    RewardDestination::Staked,
                );
                let _ = match status {
                    StakerStatus::Validator => {
                        <Module<T>>::validate(
                            T::Origin::from(Some(controller.clone()).into()),
                            Default::default(),
                        )
                    }, _ => Ok(())
                };
                let _ = <Module<T>>::delegate(
                    T::Origin::from(Some(controller.clone()).into()),
                    vec![stash.clone()]
                );
            }
        });
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

decl_event!(
    pub enum Event<T> where Balance = BalanceOf<T>, <T as frame_system::Config>::AccountId {
        /// The era payout has been set; the first balance is the validator-payout; the second is
        /// the remainder from the maximum amount of reward.
        /// \[era_index, validator_payout, remainder\]
        EraPayout(EraIndex, Balance, Balance),
        /// The staker has been rewarded by this amount. \[stash, amount\]
        Reward(AccountId, Balance),
        /// One validator (and its nominators) has been slashed by the given amount.
        /// \[validator, amount\]
        Slash(AccountId, Balance),
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
        Bonded(AccountId, Balance),
        /// An account has unbonded this amount. \[stash, amount\]
        Unbonded(AccountId, Balance),
        /// An account has called `withdraw_unbonded` and removed unbonding chunks worth `Balance`
        /// from the unlocking queue. \[stash, amount\]
        Withdrawn(AccountId, Balance),
        /// A nominator has been kicked from a validator. \[nominator, stash\]
        Kicked(AccountId, AccountId),
        /// Delegated to a candivalidator
        Delegated(AccountId),
        /// Undelegate from a candivalidator
        UnDelegated(AccountId),
        PocReward(AccountId, Balance),
        /// The delegator  has been rewarded by this amount. \[accountid, amount\]
        DelegatorReward(AccountId, Balance),
        /// The validator  has been rewarded by this amount. \[accountid, amount\]
        ValidatorReward(AccountId, Balance),
    }
);

decl_error! {
    /// Error for the staking module.
    pub enum Error for Module<T: Config> {
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
        /// Invalid number of nominations.
        InvalidNumberOfNominations,
        /// Items are not sorted and unique.
        NotSortedAndUnique,
        /// Rewards for this era have already been claimed for this validator.
        AlreadyClaimed,
        /// The submitted result is received out of the open window.
        OffchainElectionEarlySubmission,
        /// The submitted result is not as good as the one stored on chain.
        OffchainElectionWeakSubmission,
        /// The snapshot data of the current window is missing.
        SnapshotUnavailable,
        /// Incorrect number of winners were presented.
        OffchainElectionBogusWinnerCount,
        /// One of the submitted winners is not an active candidate on chain (index is out of range
        /// in snapshot).
        OffchainElectionBogusWinner,
        /// Error while building the assignment type from the compact. This can happen if an index
        /// is invalid, or if the weights _overflow_.
        OffchainElectionBogusCompact,
        /// One of the submitted nominators is not an active nominator on chain.
        OffchainElectionBogusNominator,
        /// One of the submitted nominators has an edge to which they have not voted on chain.
        OffchainElectionBogusNomination,
        /// One of the submitted nominators has an edge which is submitted before the last non-zero
        /// slash of the target.
        OffchainElectionSlashedNomination,
        /// A self vote must only be originated from a validator to ONLY themselves.
        OffchainElectionBogusSelfVote,
        /// The submitted result has unknown edges that are not among the presented winners.
        OffchainElectionBogusEdge,
        /// The claimed score does not match with the one computed from the data.
        OffchainElectionBogusScore,
        /// The election size is invalid.
        OffchainElectionBogusElectionSize,
        /// The call is not allowed at the given time due to restrictions of election period.
        CallNotAllowed,
        /// Incorrect previous history depth input provided.
        IncorrectHistoryDepth,
        /// Incorrect number of slashing spans provided.
        IncorrectSlashingSpans,
        /// Internal state has become somehow corrupted and the operation cannot continue.
        BadState,
        /// Too many nomination targets supplied.
        TooManyTargets,
        /// A nomination target was supplied that was blocked or otherwise not a validator.
        BadTarget,
        /// Already delegate to a validator
        AlreadyDelegated,
        /// Have not been delegated to a validator
        NotDelegate,
        /// Credit score of delegator is too low
        CreditScoreTooLow,
        /// Target of delegation is not in candidate validators
        NotInCandidateValidator,
        /// Select too many candidate validators
        SelectTooManyValidators,
        /// No candidate validator has been selected
        SelectNoValidator,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        /// Number of sessions per era.
        const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

        /// Number of eras that staked funds must remain bonded for.
        const BondingDuration: EraIndex = T::BondingDuration::get();

        /// Number of eras that slashes are deferred by, after computation.
        ///
        /// This should be less than the bonding duration.
        /// Set to 0 if slashes should be applied immediately, without opportunity for
        /// intervention.
        const SlashDeferDuration: EraIndex = T::SlashDeferDuration::get();

        /// The number of blocks before the end of the era from which election submissions are allowed.
        ///
        /// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
        /// be used.
        ///
        /// This is bounded by being within the last session. Hence, setting it to a value more than the
        /// length of a session will be pointless.
        const ElectionLookahead: T::BlockNumber = T::ElectionLookahead::get();

        /// Maximum number of balancing iterations to run in the offchain submission.
        ///
        /// If set to 0, balance_solution will not be executed at all.
        const MaxIterations: u32 = T::MaxIterations::get();

        /// The threshold of improvement that should be provided for a new solution to be accepted.
        const MinSolutionScoreBump: Perbill = T::MinSolutionScoreBump::get();

        /// The maximum number of nominators rewarded for each validator.
        ///
        /// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
        /// their reward. This used to limit the i/o cost for the nominator payout.
        const MaxNominatorRewardedPerValidator: u32 = T::MaxNominatorRewardedPerValidator::get();

        type Error = Error<T>;

        fn deposit_event() = default;

        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            if StorageVersion::get() == Releases::V4_0_0 {
                StorageVersion::put(Releases::V5_0_0);
                migrations::migrate_to_blockable::<T>()
            } else {
                0
            }
        }

        /// sets `ElectionStatus` to `Open(now)` where `now` is the block number at which the
        /// election window has opened, if we are at the last session and less blocks than
        /// `T::ElectionLookahead` is remaining until the next new session schedule. The offchain
        /// worker, if applicable, will execute at the end of the current block, and solutions may
        /// be submitted.
        fn on_initialize(now: T::BlockNumber) -> Weight {
            T::DbWeight::get().reads_writes(4, 0)
        }

        fn on_finalize() {
            // Set the start of the first era.
            if let Some(mut active_era) = Self::active_era() {
                if active_era.start.is_none() {
                    let now_as_millis_u64 = T::UnixTime::now().as_millis().saturated_into::<u64>();
                    active_era.start = Some(now_as_millis_u64);
                    // This write only ever happens once, we don't include it in the weight in general
                    ActiveEra::put(active_era);
                }
            }
            // `on_finalize` weight is tracked in `on_initialize`
        }

        fn integrity_test() {
            sp_io::TestExternalities::new_empty().execute_with(||
                assert!(
                    T::SlashDeferDuration::get() < T::BondingDuration::get() || T::BondingDuration::get() == 0,
                    "As per documentation, slash defer duration ({}) should be less than bonding duration ({}).",
                    T::SlashDeferDuration::get(),
                    T::BondingDuration::get(),
                )
            );

            use sp_runtime::UpperOf;
            // see the documentation of `Assignment::try_normalize`. Now we can ensure that this
            // will always return `Ok`.
            // 1. Maximum sum of Vec<ChainAccuracy> must fit into `UpperOf<ChainAccuracy>`.
            assert!(
                <usize as TryInto<UpperOf<ChainAccuracy>>>::try_into(MAX_NOMINATIONS)
                .unwrap()
                .checked_mul(<ChainAccuracy>::one().deconstruct().try_into().unwrap())
                .is_some()
            );

            // 2. Maximum sum of Vec<OffchainAccuracy> must fit into `UpperOf<OffchainAccuracy>`.
            assert!(
                <usize as TryInto<UpperOf<OffchainAccuracy>>>::try_into(MAX_NOMINATIONS)
                .unwrap()
                .checked_mul(<OffchainAccuracy>::one().deconstruct().try_into().unwrap())
                .is_some()
            );
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
        #[weight = T::WeightInfo::bond()]
        pub fn bond(origin,
            controller: <T::Lookup as StaticLookup>::Source,
            #[compact] value: BalanceOf<T>,
            payee: RewardDestination<T::AccountId>,
        ) {
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

            system::Module::<T>::inc_consumers(&stash).map_err(|_| Error::<T>::BadState)?;

            // You're auto-bonded forever, here. We might improve this by only bonding when
            // you actually validate/nominate and remove once you unbond __everything__.
            <Bonded<T>>::insert(&stash, &controller);
            <Payee<T>>::insert(&stash, payee);

            let current_era = CurrentEra::get().unwrap_or(0);
            let history_depth = Self::history_depth();
            let last_reward_era = current_era.saturating_sub(history_depth);

            let stash_balance = T::Currency::free_balance(&stash);
            let value = value.min(stash_balance);
            Self::deposit_event(RawEvent::Bonded(stash.clone(), value));
            let item = StakingLedger {
                stash,
                total: value,
                active: value,
                unlocking: vec![],
                claimed_rewards: (last_reward_era..current_era).collect(),
            };
            Self::update_ledger(&controller, &item);
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
        #[weight = T::WeightInfo::bond_extra()]
        fn bond_extra(origin, #[compact] max_additional: BalanceOf<T>) {
            ensure!(Self::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
            let stash = ensure_signed(origin)?;

            let controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;

            let stash_balance = T::Currency::free_balance(&stash);
            if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
                let extra = extra.min(max_additional);
                ledger.total += extra;
                ledger.active += extra;
                // last check: the new active amount of ledger must be more than ED.
                ensure!(ledger.active >= T::Currency::minimum_balance(), Error::<T>::InsufficientValue);

                Self::deposit_event(RawEvent::Bonded(stash, extra));
                Self::update_ledger(&controller, &ledger);
            }
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
        #[weight = T::WeightInfo::unbond()]
        fn unbond(origin, #[compact] value: BalanceOf<T>) {
            ensure!(Self::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
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
                Self::deposit_event(RawEvent::Unbonded(ledger.stash, value));
            }
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
        /// - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators,
        ///   [Origin Account], Locks, BalanceOf stash.
        /// - Writes Each: SpanSlash * S
        /// NOTE: Weight annotation is the kill scenario, we refund otherwise.
        /// # </weight>
        #[weight = T::WeightInfo::withdraw_unbonded_kill(*num_slashing_spans)]
        fn withdraw_unbonded(origin, num_slashing_spans: u32) -> DispatchResultWithPostInfo {
            ensure!(Self::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
            let controller = ensure_signed(origin)?;
            let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let (stash, old_total) = (ledger.stash.clone(), ledger.total);
            if let Some(current_era) = Self::current_era() {
                ledger = ledger.consolidate_unlocked(current_era)
            }

            let post_info_weight = if ledger.unlocking.is_empty() && ledger.active <= T::Currency::minimum_balance() {
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
                Self::deposit_event(RawEvent::Withdrawn(stash, value));
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
        /// - Write: Nominators, Validators
        /// # </weight>
        #[weight = T::WeightInfo::validate()]
        pub fn validate(origin, prefs: ValidatorPrefs) {
            ensure!(Self::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let stash = &ledger.stash;
            <Validators<T>>::insert(stash, prefs);
            //update cadidate validators for PoC
            let candidate_validators = <Validators<T>>::iter().map(|(v, _)| v).collect::<Vec<_>>();
            <CandidateValidators<T>>::put(candidate_validators);
        }

        /// Declare no desire to either validate or nominate.
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
        /// - Write: Validators, Nominators
        /// # </weight>
        #[weight = T::WeightInfo::chill()]
        fn chill(origin) {
            ensure!(Self::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            Self::chill_stash(&ledger.stash);
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
        #[weight = T::WeightInfo::set_payee()]
        fn set_payee(origin, payee: RewardDestination<T::AccountId>) {
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            let stash = &ledger.stash;
            <Payee<T>>::insert(stash, payee);
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
        #[weight = T::WeightInfo::set_controller()]
        fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
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
        }

        /// Sets the ideal number of validators.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Weight: O(1)
        /// Write: Validator Count
        /// # </weight>
        #[weight = T::WeightInfo::set_validator_count()]
        fn set_validator_count(origin, #[compact] new: u32) {
            ensure_root(origin)?;
            ValidatorCount::put(new);
        }

        /// Increments the ideal number of validators.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Same as [`set_validator_count`].
        /// # </weight>
        #[weight = T::WeightInfo::set_validator_count()]
        fn increase_validator_count(origin, #[compact] additional: u32) {
            ensure_root(origin)?;
            ValidatorCount::mutate(|n| *n += additional);
        }

        /// Scale up the ideal number of validators by a factor.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// Same as [`set_validator_count`].
        /// # </weight>
        #[weight = T::WeightInfo::set_validator_count()]
        fn scale_validator_count(origin, factor: Percent) {
            ensure_root(origin)?;
            ValidatorCount::mutate(|n| *n += factor * *n);
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
        #[weight = T::WeightInfo::force_no_eras()]
        fn force_no_eras(origin) {
            ensure_root(origin)?;
            ForceEra::put(Forcing::ForceNone);
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
        #[weight = T::WeightInfo::force_new_era()]
        fn force_new_era(origin) {
            ensure_root(origin)?;
            ForceEra::put(Forcing::ForceNew);
        }

        /// Set the validators who cannot be slashed (if any).
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - O(V)
        /// - Write: Invulnerables
        /// # </weight>
        #[weight = T::WeightInfo::set_invulnerables(invulnerables.len() as u32)]
        fn set_invulnerables(origin, invulnerables: Vec<T::AccountId>) {
            ensure_root(origin)?;
            <Invulnerables<T>>::put(invulnerables);
        }

        /// Set the validator white list.
        /// null , any validators can come up for election; not null, only validators in whitelist can come up for election
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - O(V)
        /// - Write: ValidatorWhiteList
        /// # </weight>
        #[weight = T::WeightInfo::set_validator_whitelist(whitelist.len() as u32)]
        fn set_validator_whitelist(origin, whitelist: Vec<T::AccountId>) {
            ensure_root(origin)?;
            <ValidatorWhiteList<T>>::put(whitelist);
        }

        /// Force a current staker to become completely unstaked, immediately.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// O(S) where S is the number of slashing spans to be removed
        /// Reads: Bonded, Slashing Spans, Account, Locks
        /// Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators, Account, Locks
        /// Writes Each: SpanSlash * S
        /// # </weight>
        #[weight = T::WeightInfo::force_unstake(*num_slashing_spans)]
        fn force_unstake(origin, stash: T::AccountId, num_slashing_spans: u32) {
            ensure_root(origin)?;

            // remove all staking-related information.
            Self::kill_stash(&stash, num_slashing_spans)?;

            // remove the lock.
            T::Currency::remove_lock(STAKING_ID, &stash);
        }

        /// Force there to be a new era at the end of sessions indefinitely.
        ///
        /// The dispatch origin must be Root.
        ///
        /// # <weight>
        /// - Weight: O(1)
        /// - Write: ForceEra
        /// # </weight>
        #[weight = T::WeightInfo::force_new_era_always()]
        fn force_new_era_always(origin) {
            ensure_root(origin)?;
            ForceEra::put(Forcing::ForceAlways);
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
        #[weight = T::WeightInfo::cancel_deferred_slash(slash_indices.len() as u32)]
        fn cancel_deferred_slash(origin, era: EraIndex, slash_indices: Vec<u32>) {
            T::SlashCancelOrigin::ensure_origin(origin)?;

            ensure!(!slash_indices.is_empty(), Error::<T>::EmptyTargets);
            ensure!(is_sorted_and_unique(&slash_indices), Error::<T>::NotSortedAndUnique);

            let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);
            let last_item = slash_indices[slash_indices.len() - 1];
            ensure!((last_item as usize) < unapplied.len(), Error::<T>::InvalidSlashIndex);

            for (removed, index) in slash_indices.into_iter().enumerate() {
                let index = (index as usize) - removed;
                unapplied.remove(index);
            }

            <Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
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
        #[weight = T::WeightInfo::rebond(MAX_UNLOCKING_CHUNKS as u32)]
        fn rebond(origin, #[compact] value: BalanceOf<T>) -> DispatchResultWithPostInfo {
            ensure!(Self::era_election_status().is_closed(), Error::<T>::CallNotAllowed);
            let controller = ensure_signed(origin)?;
            let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
            ensure!(!ledger.unlocking.is_empty(), Error::<T>::NoUnlockChunk);

            let ledger = ledger.rebond(value);
            // last check: the new active amount of ledger must be more than ED.
            ensure!(ledger.active >= T::Currency::minimum_balance(), Error::<T>::InsufficientValue);

            Self::update_ledger(&controller, &ledger);
            Ok(Some(
                35 * WEIGHT_PER_MICROS
                + 50 * WEIGHT_PER_NANOS * (ledger.unlocking.len() as Weight)
                + T::DbWeight::get().reads_writes(3, 2)
            ).into())
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
        ///     - Writes Each: ErasValidatorReward, ErasRewardPoints, ErasTotalStake, ErasStartSessionIndex
        /// # </weight>
        #[weight = T::WeightInfo::set_history_depth(*_era_items_deleted)]
        fn set_history_depth(origin,
            #[compact] new_history_depth: EraIndex,
            #[compact] _era_items_deleted: u32,
        ) {
            ensure_root(origin)?;
            if let Some(current_era) = Self::current_era() {
                HistoryDepth::mutate(|history_depth| {
                    let last_kept = current_era.checked_sub(*history_depth).unwrap_or(0);
                    let new_last_kept = current_era.checked_sub(new_history_depth).unwrap_or(0);
                    for era_index in last_kept..new_last_kept {
                        Self::clear_era_information(era_index);
                    }
                    *history_depth = new_history_depth
                })
            }
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
        /// - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators, Stash Account, Locks
        /// - Writes Each: SpanSlash * S
        /// # </weight>
        #[weight = T::WeightInfo::reap_stash(*num_slashing_spans)]
        fn reap_stash(_origin, stash: T::AccountId, num_slashing_spans: u32) {
            let at_minimum = T::Currency::total_balance(&stash) == T::Currency::minimum_balance();
            ensure!(at_minimum, Error::<T>::FundedTarget);
            Self::kill_stash(&stash, num_slashing_spans)?;
            T::Currency::remove_lock(STAKING_ID, &stash);
        }

        /// delegate credit score to validators in vec equally
        #[weight = 10_000]
        pub fn delegate(origin, validators: Vec<T::AccountId>) -> DispatchResult {
            // check signature
            let controller = ensure_signed(origin)?;

            // check credit pass threshold
            if T::CreditInterface::pass_threshold(&controller, 0) == false {
                Err(Error::<T>::CreditScoreTooLow)?
            }
            // check validators size
            if validators.len() > T::MaxValidatorsCanSelected::get() {
                Err(Error::<T>::SelectTooManyValidators)?
            }
            if validators.len() == 0 {
                Err(Error::<T>::SelectNoValidator)?
            }

            // check if controller has call delegated
            if <DelegatedToValidators<T>>::contains_key(&controller){
                Err(Error::<T>::AlreadyDelegated)?
            }

            // check target validators in candidate_validators
            let candidate_validators = Self::candidate_validators().unwrap_or_default();
            for validator in validators.clone() {
                if !candidate_validators.contains(&validator){
                    Err(Error::<T>::NotInCandidateValidator)?
                }
            }

            // get avg score to validators
            let validators_vec = Self::cut_credit_score(&controller, validators.clone());
            Self::_delegate(controller.clone(), validators_vec);

            let credit_delegate_info = CreditDelegateInfo{
                delegator: controller.clone(),
                score: T::CreditInterface::get_credit_score(&controller).unwrap(),
                validators: validators.clone(),
            };
            <DelegatedToValidators<T>>::insert(controller.clone(), credit_delegate_info);

            Self::deposit_event(RawEvent::Delegated(controller));
            Ok(())
        }

        /// undelegate
        #[weight = 10_000]
        pub fn undelegate(origin) -> DispatchResult {
            let controller = ensure_signed(origin)?;

            if !<DelegatedToValidators<T>>::contains_key(&controller){
                Err(Error::<T>::NotDelegate)?
            }
            Self::_undelegate(&controller);
            <DelegatedToValidators<T>>::remove(&controller);

            Self::deposit_event(RawEvent::UnDelegated(controller));
            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    /// The total balance that can be slashed from a stash account as of right now.
    pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
        // Weight note: consider making the stake accessible through stash.
        Self::bonded(stash)
            .and_then(Self::ledger)
            .map(|l| l.active)
            .unwrap_or_default()
    }

    /// internal impl of delegated_credit_score
    fn delegated_credit_score_of_vote_weight(validator: &T::AccountId) -> VoteWeight {
        let score = Self::delegated_score_of_validator(validator).unwrap_or(0);
        let credit_to_balance = <T::CurrencyToNumber as Convert<u128, BalanceOf<T>>>::convert(
            score as u128 * T::CreditToTokenFactor::get(),
        );
        let total_credit =
            Self::total_delegated_score(Self::current_era().unwrap_or_default()).unwrap_or(0);
        let total_credit_to_balance = <T::CurrencyToNumber as Convert<u128, BalanceOf<T>>>::convert(
            total_credit as u128 * T::CreditToTokenFactor::get(),
        );
        let vote_weight = T::CurrencyToVote::to_vote(credit_to_balance, total_credit_to_balance);
        log!(
            info,
            "💸 PoC (validator {:?}  with: PoC vote weight {} and score {}).",
            validator,
            vote_weight / 2,
            score,
        );
        vote_weight
    }
    /// Internal impl of [`Self::slashable_balance_of`] that returns [`VoteWeight`].
    pub fn slashable_balance_of_vote_weight(
        stash: &T::AccountId,
        issuance: BalanceOf<T>,
    ) -> VoteWeight {
        Self::delegated_credit_score_of_vote_weight(stash)
    }

    /// Returns a closure around `slashable_balance_of_vote_weight` that can be passed around.
    ///
    /// This prevents call sites from repeatedly requesting `total_issuance` from backend. But it is
    /// important to be only used while the total issuance is not changing.
    pub fn slashable_balance_of_fn() -> Box<dyn Fn(&T::AccountId) -> VoteWeight> {
        // NOTE: changing this to unboxed `impl Fn(..)` return type and the module will still
        // compile, while some types in mock fail to resolve.
        let issuance = T::Currency::total_issuance();
        Box::new(move |who: &T::AccountId| -> VoteWeight {
            Self::slashable_balance_of_vote_weight(who, issuance)
        })
    }

    /// Dump the list of validators and nominators into vectors and keep them on-chain.
    ///
    /// This data is used to efficiently evaluate election results. returns `true` if the operation
    /// is successful.
    pub fn create_stakers_snapshot() -> (bool, Weight) {
        let mut consumed_weight = 0;
        let mut add_db_reads_writes = |reads, writes| {
            consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
        };
        let validators = <Validators<T>>::iter().map(|(v, _)| v).collect::<Vec<_>>();

        let num_validators = validators.len();
        add_db_reads_writes((num_validators) as Weight, 0);

        if num_validators > MAX_VALIDATORS {
            log!(
                warn,
                "💸 Snapshot size too big [{} <> {}].",
                num_validators,
                MAX_VALIDATORS,
            );
            (false, consumed_weight)
        } else {
            <SnapshotValidators<T>>::put(validators);
            add_db_reads_writes(0, 2);
            (true, consumed_weight)
        }
    }

    /// Clears both snapshots of stakers.
    fn kill_stakers_snapshot() {
        <SnapshotValidators<T>>::kill();
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

            match ForceEra::get() {
                Forcing::ForceNew => ForceEra::kill(),
                Forcing::ForceAlways => (),
                Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
                _ => {
                    // Either `ForceNone`, or `NotForcing && era_length < T::SessionsPerEra::get()`.
                    if era_length + 1 == T::SessionsPerEra::get() {
                        IsCurrentSessionFinal::put(true);
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

    /// Basic and cheap checks that we perform in validate unsigned, and in the execution.
    ///
    /// State reads: ElectionState, CurrentEr, QueuedScore.
    ///
    /// This function does weight refund in case of errors, which is based upon the fact that it is
    /// called at the very beginning of the call site's function.
    pub fn pre_dispatch_checks(score: ElectionScore, era: EraIndex) -> DispatchResultWithPostInfo {
        // discard solutions that are not in-time
        // check window open
        ensure!(
            Self::era_election_status().is_open(),
            Error::<T>::OffchainElectionEarlySubmission.with_weight(T::DbWeight::get().reads(1)),
        );

        // check current era.
        if let Some(current_era) = Self::current_era() {
            ensure!(
                current_era == era,
                Error::<T>::OffchainElectionEarlySubmission
                    .with_weight(T::DbWeight::get().reads(2)),
            )
        }

        // assume the given score is valid. Is it better than what we have on-chain, if we have any?
        if let Some(queued_score) = Self::queued_score() {
            ensure!(
                is_score_better(score, queued_score, T::MinSolutionScoreBump::get()),
                Error::<T>::OffchainElectionWeakSubmission.with_weight(T::DbWeight::get().reads(3)),
            )
        }

        Ok(None.into())
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
        let active_era = ActiveEra::mutate(|active_era| {
            let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
            *active_era = Some(ActiveEraInfo {
                index: new_index,
                // Set new active era start in next `on_finalize`. To guarantee usage of `Time`
                start: None,
            });
            new_index
        });

        let bonding_duration = T::BondingDuration::get();

        BondedEras::mutate(|bonded| {
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

    /// distribute pocr reward
    fn distribute_pocr_reward(current_era: EraIndex) {
        // distribute delegators
        let mut total_poc_reward = BalanceOf::<T>::zero();
        for (delegator, flag) in Delegators::<T>::iter_prefix(current_era) {
            if flag && T::NodeInterface::im_offline(&delegator) == false {
                if let Some((daily_referee_reward, daily_poc_reward)) =
                    T::CreditInterface::get_reward(&delegator)
                {
                    // update RewardData
                    if Reward::<T>::contains_key(&delegator) {
                        Reward::<T>::mutate(&delegator, |data| match data {
                            Some(reward_data) => {
                                reward_data.received_referee_reward += daily_referee_reward;
                                reward_data.daily_referee_reward = daily_referee_reward;
                                reward_data.received_pocr_reward += daily_poc_reward;
                                reward_data.daily_poc_reward = daily_poc_reward;
                            }
                            _ => (),
                        });
                    } else {
                        let reward_data = RewardData::<BalanceOf<T>> {
                            total_referee_reward: T::CreditInterface::get_top_referee_reward(
                                &delegator,
                            )
                            .unwrap_or_default(),
                            received_referee_reward: daily_referee_reward,
                            daily_referee_reward: daily_referee_reward,
                            received_pocr_reward: daily_poc_reward,
                            daily_poc_reward: daily_poc_reward,
                        };
                        Reward::<T>::insert(&delegator, reward_data);
                    }
                    total_poc_reward += daily_poc_reward;
                    T::Currency::deposit_creating(
                        &delegator,
                        daily_referee_reward + daily_poc_reward,
                    );
                    Self::deposit_event(RawEvent::DelegatorReward(
                        delegator,
                        daily_referee_reward + daily_poc_reward,
                    ));
                }
            }
        }

        // distribute pocr reward to validators
        if let Some(current_era_validators) = Self::current_era_validators() {
            let len = current_era_validators.len();
            let validator_reward = Percent::from_percent(5)
                * (Perbill::from_rational_approximation(1, len as u32) * total_poc_reward);
            for (validator, _) in SelectedDelegators::<T>::iter_prefix(current_era) {
                if let Some(imbalance) = Self::make_validator_payout(&validator, validator_reward) {
                    Self::deposit_event(RawEvent::Reward(validator.clone(), imbalance.peek()));
                }
                Self::deposit_event(RawEvent::ValidatorReward(validator, validator_reward));
            }
        }
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

    /// Compute payout for era.
    fn end_era(active_era: ActiveEraInfo, _session_index: SessionIndex) {
        // Note: active_era_start can be None if end era is called during genesis config.
        if let Some(_active_era_start) = active_era.start {
            Self::check_and_adjust_delegated_score();
            Self::distribute_pocr_reward(active_era.index);
        }
    }

    /// Plan a new era. Return the potential new staking set.
    fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        // Increment or set current era.
        let current_era = CurrentEra::mutate(|s| {
            *s = Some(s.map(|s| s + 1).unwrap_or(0));
            s.unwrap()
        });
        ErasStartSessionIndex::insert(&current_era, &start_session_index);

        // Clean old era information.
        if let Some(old_era) = current_era.checked_sub(Self::history_depth() + 1) {
            Self::clear_era_information(old_era);
        }

        // Set staking information for new era.

        let maybe_new_validators = Self::select_and_update_validators(current_era);
        Self::set_current_era_validators(maybe_new_validators.clone().unwrap_or_default());

        maybe_new_validators
    }

    /// Remove all the storage items associated with the election.
    fn close_election_window() {
        // Close window.
        <EraElectionStatus<T>>::put(ElectionStatus::Closed);
        // Kill snapshots.
        Self::kill_stakers_snapshot();
        // Don't track final session.
        IsCurrentSessionFinal::put(false);
    }

    /// Select the new validator set at the end of the era.
    ///
    /// Runs [`try_do_phragmen`] and updates the following storage items:
    /// - [`EraElectionStatus`]: with `None`.
    /// - [`ErasStakers`]: with the new staker set.
    /// - [`ErasStakersClipped`].
    /// - [`ErasValidatorPrefs`].
    /// - [`ErasTotalStake`]: with the new total stake.
    /// - [`SnapshotValidators`] and [`SnapshotNominators`] are both removed.
    ///
    /// Internally, [`QueuedElected`], snapshots and [`QueuedScore`] are also consumed.
    ///
    /// If the election has been successful, It passes the new set upwards.
    ///
    /// This should only be called at the end of an era.
    fn select_and_update_validators(current_era: EraIndex) -> Option<Vec<T::AccountId>> {
        let validator_whitelist = <ValidatorWhiteList<T>>::get();
        let candidate_validators = <CandidateValidators<T>>::get().unwrap();
        let mut candidate_delegators: Vec<(T::AccountId, u32)> = CandidateDelegators::<T>::iter()
            .filter(|(validator, _)| {
                (validator_whitelist.len() == 0 || validator_whitelist.contains(&validator))
                && candidate_validators.contains(&validator)
            })
            .map(|(validator, delegators)| (validator, delegators.len() as u32))
            .collect();
        

        if candidate_delegators.len() < Self::minimum_validator_count().max(1) as usize {
            // If we don't have enough candidates, nothing to do.
            log!(
                warn,
                "💸 Chain does not have enough staking candidates to operate. Era {:?}.",
                Self::current_era()
            );
            None
        } else {
            candidate_delegators.sort_by(|a, b| a.1.cmp(&b.1).reverse());
            candidate_delegators.truncate(Self::validator_count() as usize);
            let elected_validators: Vec<T::AccountId> =
                candidate_delegators.iter().map(|(v, _)| (*v).clone()).collect();
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
                let stake = Self::bonded(&v)
                    .and_then(Self::ledger)
                    .map(|l| l.active)
                    .unwrap_or_default();
                let exposure = Exposure {
                    total: stake,
                    own: stake,
                    others: vec![],
                };
                ErasStakers::<T>::insert(current_era, &v, &exposure);
                total_stake = total_stake.saturating_add(stake);
            }
            ErasTotalStake::<T>::insert(&current_era, total_stake);
            Some(elected_validators)
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

        system::Module::<T>::dec_consumers(stash);

        Ok(())
    }

    /// Clear all era information for given era.
    fn clear_era_information(era_index: EraIndex) {
        <ErasStakers<T>>::remove_prefix(era_index);
        <ErasStakersClipped<T>>::remove_prefix(era_index);
        <ErasValidatorPrefs<T>>::remove_prefix(era_index);
        <ErasValidatorReward<T>>::remove(era_index);
        <ErasRewardPoints<T>>::remove(era_index);
        <ErasTotalStake<T>>::remove(era_index);
        ErasStartSessionIndex::remove(era_index);
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
                for (validator, points) in validators_points.into_iter() {
                    *era_rewards.individual.entry(validator).or_default() += points;
                    era_rewards.total += points;
                }
            });
        }
    }

    /// Ensures that at the end of the current session there will be a new era.
    fn ensure_new_era() {
        match ForceEra::get() {
            Forcing::ForceAlways | Forcing::ForceNew => (),
            _ => ForceEra::put(Forcing::ForceNew),
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
        SlashRewardFraction::put(fraction);
    }

    fn _delegate(delegator: T::AccountId, validators_vec: Vec<(T::AccountId, u64)>) {
        for (validator, score) in validators_vec {
            let has_delegated =
                <HasDelegatedToValidator<T>>::get(&delegator, &validator).unwrap_or(false);
            if has_delegated {
                // delegated, update score
                let delegators = CandidateDelegators::<T>::take(&validator);
                let next_delegators: Vec<_> = delegators
                    .iter()
                    .map(|(d, s)| {
                        if *d == delegator.clone() {
                            ((*d).clone(), score)
                        } else {
                            ((*d).clone(), *s)
                        }
                    })
                    .collect();
                CandidateDelegators::<T>::insert(&validator, next_delegators);
            } else {
                // delegate first time
                if CandidateDelegators::<T>::contains_key(&validator) {
                    let mut delegators = CandidateDelegators::<T>::take(&validator);
                    delegators.push((delegator.clone(), score));
                    CandidateDelegators::<T>::insert(&validator, delegators);
                } else {
                    let delegators = vec![(delegator.clone(), score)];
                    CandidateDelegators::<T>::insert(&validator, delegators);
                }
            }
            <HasDelegatedToValidator<T>>::insert(&delegator, &validator, true);
        }
    }

    fn _undelegate(delegator: &T::AccountId) {
        for (validator, _) in HasDelegatedToValidator::<T>::iter_prefix(&delegator) {
            if CandidateDelegators::<T>::contains_key(&validator) {
                let delegators = CandidateDelegators::<T>::take(&validator);
                let next_delegators: Vec<_> = delegators
                    .iter()
                    .filter(|(d, _)| *d != delegator.clone())
                    .collect();
                CandidateDelegators::<T>::insert(&validator, next_delegators);
                <HasDelegatedToValidator<T>>::insert(&delegator, &validator, false);
            }
        }
    }

    // partion credit score of delegator
    fn cut_credit_score(
        delegator: &T::AccountId,
        target_validators: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, u64)> {
        let total_score = T::CreditInterface::get_credit_score(delegator).unwrap();
        let len = target_validators.len();
        let answer: u64 = total_score / len as u64;
        let mut remainder: u64 = total_score % len as u64;
        let mut validators: Vec<(T::AccountId, u64)> = vec![];
        for v in target_validators {
            if remainder != 0 {
                validators.push((v, answer + 1));
                remainder -= 1;
            } else {
                validators.push((v, answer));
            }
        }
        log!(
            info,
            "score of: {:?} is {}, delegate to validaors{:?}",
            delegator,
            total_score,
            validators.clone()
        );
        validators
    }

    fn check_and_adjust_delegated_score() {
        for (delegator, credit_delegate_info) in DelegatedToValidators::<T>::iter() {
            // check validators in CandidateValidators
            let target_validators = credit_delegate_info.validators;
            let mut target_is_changed = false;
            let candidate_validators = <CandidateValidators<T>>::get().unwrap();
            let next_target_validators: Vec<_> = target_validators
                .iter()
                .filter(|v| {
                    if candidate_validators.contains(v) {
                        true
                    } else {
                        target_is_changed = true;
                        false
                    }
                })
                .map(|v| (*v).clone())
                .collect();

            // score to low or target_validators not in <CandidateValidators<T>>
            if T::CreditInterface::pass_threshold(&delegator, 0) == false
                || next_target_validators.len() == 0
            {
                Self::_undelegate(&delegator);
                <DelegatedToValidators<T>>::remove(&delegator);
            } else {
                let total_score = T::CreditInterface::get_credit_score(&delegator).unwrap();
                // score has update or target_validators changed
                if total_score != credit_delegate_info.score || target_is_changed {
                    Self::_undelegate(&delegator);

                    let validators_vec = Self::cut_credit_score(&delegator, next_target_validators);
                    Self::_delegate(delegator.clone(), validators_vec);

                    let mut info = <DelegatedToValidators<T>>::take(&delegator);
                    info.score = total_score;
                    <DelegatedToValidators<T>>::insert(&delegator, info);
                }
            }
        }
    }

    fn set_current_era_validators(validators: Vec<T::AccountId>) {
        <CurrentEraValidators<T>>::put(validators.clone());
        let current_era = Self::current_era().unwrap_or(0);

        for validator in validators {
            let delegators = CandidateDelegators::<T>::get(&validator);
            let selected_delegators: Vec<_> = delegators
                .iter()
                .map(|(d, s)| {
                    Delegators::<T>::insert(current_era, d, true);
                    (d, s, false)
                })
                .collect();
            SelectedDelegators::<T>::insert(current_era, validator, selected_delegators);
        }
    }

    fn delegated_score_of_validator(validator: &T::AccountId) -> Option<u64> {
        if <CandidateDelegators<T>>::contains_key(validator) {
            let delegators = <CandidateDelegators<T>>::get(validator);
            let mut score: u64 = 0;
            for (_, s) in delegators {
                score += s;
            }
            Some(score)
        } else {
            Some(0)
        }
    }

    fn total_delegated_score(era_index: EraIndex) -> Option<u64> {
        // check delegators credit score
        Self::check_and_adjust_delegated_score();

        let mut total_score: u64 = 0;
        for (validator, _) in SelectedDelegators::<T>::iter_prefix(era_index) {
            total_score += Self::get_total_validator_score(era_index, validator).unwrap_or(0);
        }
        Some(total_score)
    }

    fn get_total_validator_score(era_index: EraIndex, validator: T::AccountId) -> Option<u64> {
        if <SelectedDelegators<T>>::contains_key(era_index, validator.clone()) {
            let delegators = <SelectedDelegators<T>>::get(era_index, validator);
            let mut total_score: u64 = 0;
            for (_, s, _) in delegators {
                total_score += s;
            }
            Some(total_score)
        } else {
            Some(0)
        }
    }

    // poc credit slash
    fn poc_slash(validator: &T::AccountId, era_index: EraIndex) {
        if <SelectedDelegators<T>>::contains_key(era_index, &validator) {
            let delegators = <SelectedDelegators<T>>::take(era_index, validator);
            let update_delegators: Vec<_> = delegators
                .iter()
                .map(|(d, s, slashed)| {
                    if *slashed == false && <DelegatedToValidators<T>>::contains_key(d) {
                        T::CreditInterface::slash_credit(d);
                        if T::CreditInterface::pass_threshold(d, 0) == false {
                            Self::_undelegate(d);
                            <DelegatedToValidators<T>>::remove(d);
                        }
                        ((*d).clone(), *s, true)
                    } else {
                        ((*d).clone(), *s, *slashed)
                    }
                })
                .collect();
            <SelectedDelegators<T>>::insert(era_index, validator, update_delegators);
        }
    }
}

/// In this implementation `new_session(session)` must be called before `end_session(session-1)`
/// i.e. the new session must be planned before the ending of the previous session.
///
/// Once the first new_session is planned, all session must start and then end in order, though
/// some session can lag in between the newest session planned and the latest session started.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Module<T> {
    fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        frame_support::debug::native::trace!(
            target: LOG_TARGET,
            "[{}] planning new_session({})",
            <frame_system::Module<T>>::block_number(),
            new_index
        );
        Self::new_session(new_index)
    }
    fn start_session(start_index: SessionIndex) {
        frame_support::debug::native::trace!(
            target: LOG_TARGET,
            "[{}] starting start_session({})",
            <frame_system::Module<T>>::block_number(),
            start_index
        );
        Self::start_session(start_index)
    }
    fn end_session(end_index: SessionIndex) {
        frame_support::debug::native::trace!(
            target: LOG_TARGET,
            "[{}] ending end_session({})",
            <frame_system::Module<T>>::block_number(),
            end_index
        );
        Self::end_session(end_index)
    }
}

impl<T: Config> historical::SessionManager<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>>
    for Module<T>
{
    fn new_session(
        new_index: SessionIndex,
    ) -> Option<Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>> {
        let candidate_validators = <Validators<T>>::iter().map(|(v, _)| v).collect::<Vec<_>>();
        <CandidateValidators<T>>::put(candidate_validators);
        <Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
            let current_era = Self::current_era()
                // Must be some as a new era has been created.
                .unwrap_or(0);

            validators
                .into_iter()
                .map(|v| {
                    let exposure = Self::eras_stakers(current_era, &v); // TODO investigate inpact
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
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T>
where
    T: Config + pallet_authorship::Config + pallet_session::Config,
{
    fn note_author(author: T::AccountId) {
        Self::reward_by_ids(vec![(author, 20)])
    }
    fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
        Self::reward_by_ids(vec![
            (<pallet_authorship::Module<T>>::author(), 2),
            (author, 1),
        ])
    }
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
    fn convert(controller: T::AccountId) -> Option<T::AccountId> {
        <Module<T>>::ledger(&controller).map(|l| l.stash)
    }
}

/// A typed conversion from stash account ID to the active exposure of nominators
/// on that account.
///
/// Active exposure is the exposure of the validator set currently validating, i.e. in
/// `active_era`. It can differ from the latest planned exposure in `current_era`.
pub struct ExposureOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
    for ExposureOf<T>
{
    fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
        if let Some(active_era) = <Module<T>>::active_era() {
            Some(<Module<T>>::eras_stakers(active_era.index, &validator))
        } else {
            None
        }
    }
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Config>
    OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>, Weight>
    for Module<T>
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
    ) -> Result<Weight, ()> {
        if !Self::can_report() {
            return Err(());
        }

        let reward_proportion = SlashRewardFraction::get();
        let mut consumed_weight: Weight = 0;
        let mut add_db_reads_writes = |reads, writes| {
            consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
        };

        let active_era = {
            let active_era = Self::active_era();
            add_db_reads_writes(1, 0);
            if active_era.is_none() {
                // this offence need not be re-submitted.
                return Ok(consumed_weight);
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
            let eras = BondedEras::get();
            add_db_reads_writes(1, 0);

            // reverse because it's more likely to find reports from recent eras.
            match eras
                .iter()
                .rev()
                .filter(|&&(_, ref sesh)| sesh <= &slash_session)
                .next()
            {
                Some(&(ref slash_era, _)) => *slash_era,
                // before bonding period. defensive - should be filtered out.
                None => return Ok(consumed_weight),
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

            // poc_stash(stash,slash_era): check delegators behind stash
            Self::poc_slash(stash, slash_era);

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
                let nominators_len = unapplied.others.len() as u64;
                let reporters_len = details.reporters.len() as u64;

                {
                    let upper_bound = 1 /* Validator/NominatorSlashInEra */ + 2 /* fetch_spans */;
                    let rw = upper_bound + nominators_len * upper_bound;
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
                            (1 + nominators_len) * slash_cost.0 + reward_cost.0 * reporters_len,
                            (1 + nominators_len) * slash_cost.1 + reward_cost.1 * reporters_len,
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

        Ok(consumed_weight)
    }

    fn can_report() -> bool {
        Self::era_election_status().is_closed()
    }
}

/// Filter historical offences out and only allow those from the bonding period.
pub struct FilterHistoricalOffences<T, R> {
    _inner: sp_std::marker::PhantomData<(T, R)>,
}

impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O>
    for FilterHistoricalOffences<Module<T>, R>
where
    T: Config,
    R: ReportOffence<Reporter, Offender, O>,
    O: Offence<Offender>,
{
    fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), OffenceError> {
        // disallow any slashing from before the current bonding period.
        let offence_session = offence.session_index();
        let bonded_eras = BondedEras::get();

        if bonded_eras
            .first()
            .filter(|(_, start)| offence_session >= *start)
            .is_some()
        {
            R::report_offence(reporters, offence)
        } else {
            <Module<T>>::deposit_event(RawEvent::OldSlashingReportDiscarded(offence_session));
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
