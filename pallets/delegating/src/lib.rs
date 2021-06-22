#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use sp_runtime::Perbill;
use sp_std::vec::Vec;

pub trait CreditDelegateInterface<AccountId, B, PB> {
    fn set_current_era(current_era: EraIndex);
    fn set_current_era_validators(validators: Vec<AccountId>);
    fn set_candidate_validators(candidate_validators: Vec<AccountId>);

    /// obtain the total delegated score of accountid in current era
    fn delegated_score_of_validator(validator: &AccountId) -> Option<u64>;

    /// obtain the total delegated score of accountid in current era
    fn total_delegated_score(era_index: EraIndex) -> Option<u64>;

    fn get_total_validator_score(era_index: EraIndex, validator: AccountId) -> Option<u64>;

    fn set_eras_reward(era_index: EraIndex, total_reward: B);

    fn payout_delegators(
        era_index: EraIndex,
        commission: Perbill,
        validator_reward_part: Perbill,
        validator: AccountId,
        validator_payee: AccountId,
    ) -> bool;
    fn make_payout(receiver: AccountId, amount: B) -> Option<PB>;

    fn poc_slash(validator: &AccountId, era_index: EraIndex);
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use log::error;

    use frame_support::codec::{Decode, Encode};
    use frame_support::sp_runtime::Perbill;
    use frame_support::traits::{Currency, Get, Imbalance, LockableCurrency};
    use frame_system::ensure_signed;
    use pallet_credit::CreditInterface;
    use sp_std::vec;
    use super::*;

    pub(crate) const LOG_TARGET: &'static str = "credit";

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

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type CreditInterface: CreditInterface<Self::AccountId>;
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
        /// max validators can be selected to delegate
        type MaxValidatorsCanSelected: Get<usize>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    type PositiveImbalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance;

    pub type EraIndex = u32;

    #[derive(Decode, Encode, Default)]
    pub struct CreditDelegateInfo<AccountId> {
        pub delegator: AccountId,
        pub score: u64,
        pub validators: Vec<AccountId>,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// (delegator) -> CreditDelegateInfo{}
    #[pallet::storage]
    #[pallet::getter(fn delegated_to_validators)]
    pub(super) type DelegatedToValidators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditDelegateInfo<T::AccountId>, ValueQuery>;

    /// (delegator, validator) -> bool     
    #[pallet::storage]
    #[pallet::getter(fn has_delegated_to_validator)]
    pub(super) type HasDelegatedToValidator<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    // Candidate delegators info
    // (candidateValidator) -> Vec<(delegator, score)>
    #[pallet::storage]
    #[pallet::getter(fn candidate_delegators)]
    pub(super) type CandidateDelegators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(T::AccountId, u64)>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn eras_validator_reward)]
    pub(super) type ErasValidatorReward<T: Config> =
        StorageMap<_, Blake2_128Concat, EraIndex, BalanceOf<T>, OptionQuery>;

    /// (EraIndex, validatorId) -> Vec<(delegator, score, hasSlashed)>    
    #[pallet::storage]
    #[pallet::getter(fn selected_delegators)]
    pub(super) type SelectedDelegators<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, EraIndex, Blake2_128Concat, T::AccountId, Vec<(T::AccountId, u64, bool)>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_era)]
    pub type CurrentEra<T> = StorageValue<_, EraIndex, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_era_validators)]
    pub type CurrentEraValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, OptionQuery>;

    /// candidate validator list
    #[pallet::storage]
    #[pallet::getter(fn get_candidate_validators)]
    pub type CandidateValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, OptionQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Delegated(T::AccountId),
        UnDelegated(T::AccountId),
        WithdrawCredit(T::AccountId, u64),
        /// The staker has been rewarded by this amount. \[stash, amount\]
        Reward(T::AccountId, BalanceOf<T>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        AlreadyDelegated,
        NotDelegate,
        CreditLocked,
        NoCreditLedgerData,
        NotRightEra,
        CreditScoreTooLow,
        NonCandidateValidator,
        NotInCandidateValidator,
        SelectTooManyValidators,
        SelectNoValidator,
        DelegateMoreCreditScoreThanOwn,
        InvalidEraToReward,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// delegate credit score to validators in vec equally
        #[pallet::weight(10_000)]
        pub fn delegate(origin: OriginFor<T>, validators: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
            // check signature
            let controller = ensure_signed(origin)?;

            // check credit pass threshold
            if T::CreditInterface::pass_threshold(&controller, 0) == false {
                error!("Credit score is to low to delegating a validator!");
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
            let candidate_validators = <CandidateValidators<T>>::get().unwrap();
            for validator in validators.clone() {
                if !candidate_validators.contains(&validator){
                    error!("Validator AccountId  isn't in candidateValidators");
                    Err(Error::<T>::NotInCandidateValidator)?
                }
            }

            // get avg score to validators
            let validators_vec = Self::cut_credit_score(controller.clone(), validators.clone());
            Self::_delegate(controller.clone(), validators_vec);

            let credit_delegate_info = CreditDelegateInfo{
                delegator: controller.clone(),
                score: T::CreditInterface::get_credit_score(controller.clone()).unwrap(),
                validators: validators.clone(),
            };
            <DelegatedToValidators<T>>::insert(controller.clone(), credit_delegate_info);

            Self::deposit_event(Event::Delegated(controller));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn undelegate(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let controller = ensure_signed(origin)?;

            if !<DelegatedToValidators<T>>::contains_key(controller.clone()){
                Err(Error::<T>::NotDelegate)?
            }
            Self::_undelegate(controller.clone());
            <DelegatedToValidators<T>>::remove(controller.clone());

            Self::deposit_event(Event::UnDelegated(controller));
            Ok(().into())
        }
    }

    impl<T: Config> Module<T> {
        fn _delegate(delegator: T::AccountId, validators_vec: Vec<(T::AccountId, u64)>) {
            for (validator, score) in validators_vec {
                let has_delegated =
                    <HasDelegatedToValidator<T>>::get(delegator.clone(), validator.clone())
                        .unwrap_or(false);
                if has_delegated == false {
                    // delegate first times
                    if CandidateDelegators::<T>::contains_key(validator.clone()) {
                        let mut delegators = CandidateDelegators::<T>::take(validator.clone());
                        delegators.push((delegator.clone(), score));
                        CandidateDelegators::<T>::insert(validator.clone(), delegators);
                    } else {
                        let delegators = vec![(delegator.clone(), score)];
                        CandidateDelegators::<T>::insert(validator.clone(), delegators);
                    }
                } else {
                    // delegated, update score
                    let delegators = CandidateDelegators::<T>::take(validator.clone());
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
                    CandidateDelegators::<T>::insert(validator.clone(), next_delegators);
                }
                <HasDelegatedToValidator<T>>::insert(delegator.clone(), validator.clone(), true);
            }
        }
    
        fn _undelegate(delegator: T::AccountId) {
            for (validator, _) in HasDelegatedToValidator::<T>::iter_prefix(delegator.clone()) {
                if CandidateDelegators::<T>::contains_key(validator.clone()) {
                    let delegators = CandidateDelegators::<T>::take(validator.clone());
                    let next_delegators: Vec<_> = delegators
                        .iter()
                        .filter(|(d, _)| *d != delegator.clone())
                        .collect();
                    CandidateDelegators::<T>::insert(validator.clone(), next_delegators);
                    <HasDelegatedToValidator<T>>::insert(delegator.clone(), validator.clone(), false);
                }
            }
        }
    
        // partion credit score of delegator
        fn cut_credit_score(
            delegator: T::AccountId,
            target_validators: Vec<T::AccountId>,
        ) -> Vec<(T::AccountId, u64)> {
            let total_score = T::CreditInterface::get_credit_score(delegator.clone()).unwrap();
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
                    Self::_undelegate(delegator.clone());
                    <DelegatedToValidators<T>>::remove(delegator.clone());
                } else {
                    let total_score = T::CreditInterface::get_credit_score(delegator.clone()).unwrap();
                    // score has update or target_validators changed
                    if total_score != credit_delegate_info.score || target_is_changed == true {
                        Self::_undelegate(delegator.clone());
    
                        let validators_vec =
                            Self::cut_credit_score(delegator.clone(), next_target_validators);
                        Self::_delegate(delegator.clone(), validators_vec);
    
                        let mut info = <DelegatedToValidators<T>>::take(delegator.clone());
                        info.score = total_score;
                        <DelegatedToValidators<T>>::insert(delegator.clone(), info);
                    }
                }
            }
        }
    }
    impl<T: Config> CreditDelegateInterface<T::AccountId, BalanceOf<T>, PositiveImbalanceOf<T>> for Module<T>{
        /// called per era
        fn set_current_era(current_era: EraIndex) {
            let old_era = Self::current_era().unwrap_or(0);

            if current_era > 0 && old_era < current_era {
                <CurrentEra<T>>::put(current_era);
            }
        }

        fn set_current_era_validators(validators: Vec<T::AccountId>) {
            <CurrentEraValidators<T>>::put(validators.clone());
            let current_era = Self::current_era().unwrap_or(0);

            for validator in validators {
                let delegators = CandidateDelegators::<T>::get(validator.clone());
                let selected_delegators: Vec<_> = delegators
                    .iter()
                    .map(|(d, s)| ((*d).clone(), *s, false))
                    .collect();
                SelectedDelegators::<T>::insert(current_era, validator, selected_delegators);
            }
        }

        fn set_candidate_validators(candidate_validators: Vec<T::AccountId>) {
            <CandidateValidators<T>>::put(candidate_validators);
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

        fn set_eras_reward(era_index: EraIndex, total_reward: BalanceOf<T>) {
            <ErasValidatorReward<T>>::insert(era_index, total_reward);
        }

        fn payout_delegators(
            era_index: EraIndex,
            commission: Perbill,
            validator_reward_part: Perbill,
            validator: T::AccountId,
            validator_payee: T::AccountId,
        ) -> bool {
            if !<ErasValidatorReward<T>>::contains_key(&era_index) {
                return false;
            }
            let total_payout = <ErasValidatorReward<T>>::get(&era_index).unwrap();
            let era_payout = validator_reward_part * total_payout;

            let validator_commission_payout = commission * era_payout;
            let validator_leftover_payout = era_payout - validator_commission_payout;

            // We can now make total validator payout:
            if validator_commission_payout != <BalanceOf<T>>::default() {
                if let Some(imbalance) =
                    Self::make_payout(validator_payee.clone(), validator_commission_payout)
                {
                    Self::deposit_event(Event::Reward(validator_payee, imbalance.peek()));
                }
            }
            // Lets now calculate how this is split to the nominators.
            // Reward only the clipped exposures. Note this is not necessarily sorted.
            let era_total_score =
                Self::get_total_validator_score(era_index, validator.clone()).unwrap();
            let delegators = <SelectedDelegators<T>>::get(era_index, validator.clone());
            for (who, s, _) in delegators {
                let delegator_exposure_part = Perbill::from_rational_approximation(s, era_total_score);

                let delegator_reward: BalanceOf<T> =
                    delegator_exposure_part * validator_leftover_payout;
                // We can now make nominator payout:
                if let Some(imbalance) = Self::make_payout(who.clone(), delegator_reward) {
                    Self::deposit_event(Event::Reward(who, imbalance.peek()));
                }
            }
            true
        }

        fn make_payout(receiver: T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
            Some(T::Currency::deposit_creating(&receiver, amount))
        }

        // poc credit slash
        fn poc_slash(validator: &T::AccountId, era_index: EraIndex) {
            if <SelectedDelegators<T>>::contains_key(era_index, validator.clone()) {
                let delegators = <SelectedDelegators<T>>::take(era_index, validator);
                let update_delegators: Vec<_> = delegators
                    .iter()
                    .map(|(d, s, slashed)| {
                        if *slashed == false && <DelegatedToValidators<T>>::contains_key((*d).clone()) {
                            T::CreditInterface::credit_slash((*d).clone());
                            // undelegate
                            Self::_undelegate((*d).clone());
                            <DelegatedToValidators<T>>::remove((*d).clone());
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
}