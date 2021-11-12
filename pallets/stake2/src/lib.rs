#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode, HasCompact};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    pub use lib_staking::weights::WeightInfo;
    use lib_staking::{EraIndex, ValidatorPrefs};
    use pallet_credit::CreditInterface;
    use pallet_deeper_node::NodeInterface;
    use sp_std::{
        cmp, cmp::Ordering, collections::btree_map::BTreeMap, collections::btree_set::BTreeSet,
        convert::From, convert::TryInto, prelude::*,
    };

    #[cfg(feature = "std")]
    use frame_support::sp_runtime::{Deserialize, Serialize};
    use lib_staking::BalanceOf;

    #[macro_export]
    macro_rules! log {
		($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
			frame_support::debug::$level!(
				target: crate::LOG_TARGET,
				$patter $(, $values)*
			)
		};
	}

    #[derive(Encode, Decode, Default, RuntimeDebug)]
    pub struct RewardData<Balance: HasCompact> {
        pub total_referee_reward: Balance,
        pub received_referee_reward: Balance,
        pub referee_reward: Balance,
        pub received_pocr_reward: Balance,
        pub poc_reward: Balance,
    }

    #[derive(Decode, Encode, Default, Debug)]
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

    #[derive(Decode, Encode, Default)]
    pub struct ValidatorData<AccountId: Ord> {
        pub delegators: BTreeSet<AccountId>,
        pub elected_era: EraIndex,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + lib_staking::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// CreditInterface of credit pallet
        type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;

        /// NodeInterface of deeper-node pallet
        type NodeInterface: NodeInterface<Self::AccountId, Self::BlockNumber>;

        type MaxDelegates: Get<usize>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn something)]
    // Learn more about declaring storage items:
    // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
    pub type Something<T> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn delegators_key_prefix)]
    pub type DelegatorsKeyPrefix<T> = StorageValue<_, Vec<u8>>;

    #[pallet::storage]
    #[pallet::getter(fn delegators_last_key)]
    pub type DelegatorsLastKey<T> = StorageValue<_, Vec<u8>>;

    /// active delegator count
    #[pallet::storage]
    #[pallet::getter(fn active_delegator_count)]
    pub type ActiveDelegatorCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn delegator_count)]
    pub type DelegatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn delegators)]
    pub type Delegators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DelegatorData<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn candidate_validators)]
    pub type CandidateValidators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ValidatorData<T::AccountId>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An account has bonded this amount. \[stash, amount\]
        ///
        /// NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
        /// it will not be emitted for staking rewards when they are added to stake.
        Bonded(T::AccountId, BalanceOf<T>),
        /// An account has unbonded this amount. \[stash, amount\]
        Unbonded(T::AccountId, BalanceOf<T>),
        /// Delegated to a set of validators
        Delegated(T::AccountId, Vec<T::AccountId>),
        /// Undelegate from a validator
        UnDelegated(T::AccountId),
        /// The delegator  has been rewarded by this amount. \[account_id, amount\]
        DelegatorReward(T::AccountId, BalanceOf<T>),
        /// The validator  has been rewarded by this amount. \[account_id, amount\]
        ValidatorReward(T::AccountId, BalanceOf<T>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
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
        /// The call is not allowed at the given time due to restrictions of election period.
        CallNotAllowed,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let who = ensure_signed(origin)?;
            // Update storage.
            <Something<T>>::put(something);

            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;

            Ok(().into())
        }

        //#[pallet::weight(T::WeightInfo::delegate(1))]
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn delegate(
            origin: OriginFor<T>,
            validators: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure!(
                <lib_staking::Module<T>>::era_election_status().is_closed(),
                Error::<T>::CallNotAllowed
            );
            let delegator = ensure_signed(origin)?;

            ensure!(
                <lib_staking::Module::<T>>::validators(&delegator) == ValidatorPrefs::default(),
                Error::<T>::CallNotAllowed
            );

            let enough_credit = T::CreditInterface::pass_threshold(&delegator);
            ensure!(enough_credit, Error::<T>::CreditTooLow);

            ensure!(!validators.is_empty(), Error::<T>::NoValidators);
            // remove duplicates
            let validator_set: BTreeSet<T::AccountId> = validators.iter().cloned().collect();
            // check validators size
            ensure!(
                validator_set.len() <= T::MaxDelegates::get(),
                Error::<T>::TooManyValidators
            );
            for validator in &validator_set {
                ensure!(
                    <lib_staking::Module::<T>>::validators(&delegator) != ValidatorPrefs::default(),
                    Error::<T>::NotValidator
                );
            }

            let current_era = <lib_staking::Module<T>>::current_era().unwrap_or(0);
            if <Delegators<T>>::contains_key(&delegator) {
                let old_delegator_data = Self::delegators(&delegator);
                if !old_delegator_data.delegating {
                    // the delegator was not delegating
                    // the delegator delegates again
                    <ActiveDelegatorCount<T>>::mutate(|count| *count = count.saturating_add(1));
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
                        v.delegators.remove(&delegator)
                    });
                    if Self::candidate_validators(validator).delegators.is_empty() {
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
                <ActiveDelegatorCount<T>>::mutate(|count| *count = count.saturating_add(1));
                <DelegatorCount<T>>::mutate(|count| *count = count.saturating_add(1));
            };

            for validator in &validator_set {
                if <CandidateValidators<T>>::contains_key(validator) {
                    <CandidateValidators<T>>::mutate(validator, |v| {
                        v.delegators.insert(delegator.clone())
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

            Self::deposit_event(Event::Delegated(delegator, validators));
            Ok(().into())
        }
    }
}
