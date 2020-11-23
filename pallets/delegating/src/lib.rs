#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
use log::{error, info};

use frame_support::codec::{Decode, Encode};
use frame_support::traits::{Currency, LockableCurrency, Imbalance};
use frame_support::sp_runtime::{Perbill};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, ensure};
use frame_system::ensure_signed;
use pallet_credit::CreditInterface;
use sp_std::vec;
use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type CreditInterface: CreditInterface<Self::AccountId>;

    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;

pub type EraIndex = u32;

#[derive(Default, Clone, Encode, Decode)]
pub struct CreditScoreLedger<AccountId> {
    delegated_account: AccountId,
    delegated_score: u64,
    validator_account: AccountId,
    withdraw_era: EraIndex,
}

pub const CREDIT_LOCK_DURATION: u32 = 10; //todo

decl_storage! {
    trait Store for Module<T: Trait> as Delegating  {
        // 存质押的credit 及 delegater id
        CreditLedger get(fn credit_ledger): map hasher(blake2_128_concat) T::AccountId
            => CreditScoreLedger<T::AccountId>;
        
        // current delegators info
        CurrentDelegators get(fn current_delegators): map hasher(blake2_128_concat) T::AccountId => Vec<T::AccountId>;

        pub ErasValidatorReward get(fn eras_validator_reward):
            map hasher(twox_64_concat) EraIndex => Option<BalanceOf<T>>;

        // 存PoC生效的Era, validator id
        Delegators get(fn delegators): double_map hasher(blake2_128_concat) EraIndex,
        hasher(blake2_128_concat) T::AccountId => Vec<(T::AccountId, u64)>;

        pub CurrentEra get(fn current_era): Option<EraIndex>;
        pub CurrentEraValidators get(fn current_era_validators): Option<Vec<T::AccountId>>;

        //  candidate validator list
        pub CandidateValidators get(fn candidate_validators): Option<Vec<T::AccountId>>;

    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
         Balance = BalanceOf<T>,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        Delegated(AccountId, AccountId),
        UnDelegated(AccountId),
        WithdrawCredit(AccountId, u64),
        /// The staker has been rewarded by this amount. \[stash, amount\]
        Reward(AccountId, Balance),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        AlreadyDelegated,
        NotDelegate,
        CreditLocked,
        NoCreditLedgerData,
        NotRightEra,
        CreditScoreTooLow,
        NonCandidateValidator,
        InvalidEraToReward,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        //
        #[weight = 10_000]
        pub fn delegate(origin, validator: T::AccountId) -> dispatch::DispatchResult {
            let controller = ensure_signed(origin)?;

            if let Some(candidate_validators) = CandidateValidators::<T>::get(){
                if !candidate_validators.contains(&validator){
                    error!("Validator AccountId  isn't in candidateValidators");
                    Err(Error::<T>::NonCandidateValidator)?
                }
            }

            if T::CreditInterface::pass_threshold(controller.clone(), 0) == false{
                error!("Credit score is to low to delegating a validator!");
                Err(Error::<T>::CreditScoreTooLow)?
            }else{
                let score = T::CreditInterface::get_credit_score(controller.clone()).unwrap();
                if CreditLedger::<T>::contains_key(controller.clone()){ // change delegate target
                    let _ledger = CreditLedger::<T>::get(controller.clone());
                    if _ledger.validator_account != validator.clone(){ // target is diffent
                        CreditLedger::<T>::mutate(controller.clone(), |ledger| {
                            (*ledger).validator_account = validator.clone();
                            (*ledger).delegated_score = score;
                            (*ledger).withdraw_era = 0;
                        });
                        let last_validator = _ledger.validator_account;

                        // remove controller from old validator
                        if CurrentDelegators::<T>::contains_key(last_validator.clone()){
                            let delegators = CurrentDelegators::<T>::take(last_validator.clone());
                            let next_delegators: Vec<_> = delegators
                                .iter()
                                .filter(|delegator|{
                                    *delegator != &controller
                                })
                                .collect();
                            CurrentDelegators::<T>::insert(last_validator.clone(), next_delegators);
                        }

                        // add controller to new validator
                        if CurrentDelegators::<T>::contains_key(validator.clone()){
                            let mut delegators = CurrentDelegators::<T>::take(validator.clone());
                            delegators.push(controller.clone());
                            CurrentDelegators::<T>::insert(validator.clone(), delegators);
                        }else{
                            let delegators = vec![controller.clone()];
                            CurrentDelegators::<T>::insert(validator.clone(), delegators);
                        }

                    }else{ // target is same
                        CreditLedger::<T>::mutate(controller.clone(), |ledger| {
                            (*ledger).validator_account = validator.clone();
                            (*ledger).delegated_score = score;
                            (*ledger).withdraw_era = 0;
                        });
                    }
                }else{ // delegate
                    let ledger = CreditScoreLedger{
                        delegated_account: controller.clone(),
                        delegated_score: score,
                        validator_account: validator.clone(),
                        withdraw_era: 0,
                    };
                    CreditLedger::<T>::insert(controller.clone(), ledger.clone());

                    if CurrentDelegators::<T>::contains_key(validator.clone()){
                        let mut delegators = CurrentDelegators::<T>::take(validator.clone());
                        delegators.push(controller.clone());
                        CurrentDelegators::<T>::insert(validator.clone(), delegators);
                    }else{
                        let delegators = vec![controller.clone()];
                        CurrentDelegators::<T>::insert(validator.clone(), delegators);
                    }

                    Self::deposit_event(RawEvent::Delegated(controller, validator));
                }
            }
            Ok(())
        }

        #[weight = 10_000]
        pub fn undelegate(origin) -> dispatch::DispatchResult {
            info!("will undelegate credit score ");

            let controller = ensure_signed(origin)?;
            ensure!(CreditLedger::<T>::contains_key(controller.clone()), Error::<T>::NotDelegate);

            let ledger = CreditLedger::<T>::get(controller.clone());
            ensure!(ledger.withdraw_era == 0, Error::<T>::CreditLocked);

            let current_era = CurrentEra::get().unwrap_or(0);
            let withdraw_era = current_era + CREDIT_LOCK_DURATION;

            // delete delegator account id from CurrentDelegators
            let validator = ledger.validator_account;
            if CurrentDelegators::<T>::contains_key(&validator){
                let delegators = CurrentDelegators::<T>::take(&validator);
                let next_delegators:Vec<_> = delegators
                    .iter()
                    .filter(|delegator|{
                        *delegator != &controller
                    })
                    .collect();
                CurrentDelegators::<T>::insert(&validator, next_delegators);
            }

            // update withdraw_era 
            CreditLedger::<T>::mutate(controller.clone(),|ledger| (*ledger).withdraw_era = withdraw_era);

            Self::deposit_event(RawEvent::UnDelegated(controller));
            Ok(())
        }

        ///
        ///
        /// [storage]
        /// Read  :  Delegated, CreditLedger, CurrentEra
        /// Write :  Delegated, CreditLedger, Delegator
        ///
        #[weight = 10_000]
        pub fn withdraw_credit_score(origin) -> dispatch::DispatchResult{
            info!("withdraw credit score");

            let controller = ensure_signed(origin)?;

            // 
            if !CreditLedger::<T>::contains_key(controller.clone()) {
                error!("can't found credit ledger for your account ");
                Err(Error::<T>::NoCreditLedgerData)?
            }

            // 
            let ledger = CreditLedger::<T>::get(controller.clone());
            let current_era = CurrentEra::get().unwrap_or(0);
            if ledger.withdraw_era > current_era {
                error!("can't withdraw credit score because it's not the right time yet ");
                Err(Error::<T>::NotRightEra)?
            }

            // 
            CreditLedger::<T>::remove(controller.clone());

            Self::deposit_event(RawEvent::WithdrawCredit(controller, ledger.delegated_score));

            Ok(())
        }

        ///	delegate after calling undelegate()
        #[weight = 10_000]
        pub fn redelegate(origin) -> dispatch::DispatchResult {
            let controller = ensure_signed(origin)?;
            ensure!(CreditLedger::<T>::contains_key(controller.clone()), Error::<T>::NotDelegate);
            ensure!(T::CreditInterface::pass_threshold(controller.clone(), 0), Error::<T>::CreditScoreTooLow);
            let score = T::CreditInterface::get_credit_score(controller.clone()).unwrap();

            CreditLedger::<T>::mutate(controller.clone(),|ledger| {
                (*ledger).withdraw_era = 0;
                (*ledger).delegated_score = score;
            });

            // update Delegators for that the era has changed between "undelegate" and "redelegate"
            let ledger = CreditLedger::<T>::get(controller.clone());
            let validator = ledger.validator_account;

            if CurrentDelegators::<T>::contains_key(validator.clone()){
                let mut delegators = CurrentDelegators::<T>::take(validator.clone());
                delegators.push(controller.clone());
                CurrentDelegators::<T>::insert(validator.clone(), delegators);
            }else{
                let delegators = vec![controller.clone()];
                CurrentDelegators::<T>::insert(validator.clone(), delegators);
            }

            Ok(())
        }

    }
}

pub trait CreditDelegateInterface<AccountId, B, PB> {
    fn set_current_era(current_era: EraIndex);
    fn set_current_era_validators(validators: Vec<AccountId>);
    fn set_candidate_validators(candidate_validators: Vec<AccountId>);

    /// 获取当前ERA accountid代理的总分
    fn delegated_score_of_validator(validator: &AccountId) -> Option<u64>;

    /// 获取当前ERA代理的总分
    fn total_delegated_score() -> Option<u64>;

    fn get_delegated_score(account_id: AccountId) -> Option<u64>;

    /// kill delegator's credit score
    fn kill_credit(account_id: AccountId) -> bool;

    fn get_total_validator_score(era_index: EraIndex, validator: AccountId) -> Option<u64>;

    fn set_eras_reward(era_index: EraIndex, total_reward:B);

    fn payout_delegators(era_index: EraIndex, commission: Perbill, validator: AccountId,
                         validator_payee: AccountId)  -> bool;
    fn make_payout(stash: AccountId, amount: B) -> Option<PB>;
}


impl<T: Trait> CreditDelegateInterface<T::AccountId, BalanceOf<T>, PositiveImbalanceOf<T>> for Module<T> {
    /// called per era
    fn set_current_era(current_era: EraIndex) {
        let old_era = Self::current_era().unwrap_or(0);

        if current_era > 0 && old_era < current_era {
            <CurrentEra>::put(current_era);
            // 更新潜在validator背后质押credit的账户、及era_index
            if let Some(candidate_validators) = <CandidateValidators<T>>::get() {
                //let mut  total_score:u64 = 0;
                for candidate_validator in candidate_validators {
                    let delegators = <CurrentDelegators<T>>::get(candidate_validator.clone());
                    let next_delegators: Vec<_> = delegators
                        .iter()
                        .filter(|delegator| {
                            T::CreditInterface::pass_threshold((*delegator).clone(), 0)
                        })
                        .map(|delegator| {
                            let score = T::CreditInterface::get_credit_score((*delegator).clone()).unwrap();
                            //total_score = total_score + score;
                            (
                                delegator,
                                score,
                            )
                        })
                        .collect();

                    <Delegators<T>>::insert(
                        current_era,
                        candidate_validator,
                        next_delegators.clone(),
                    );
                }
            }
        }
    }

    fn set_current_era_validators(validators: Vec<T::AccountId>) {
        <CurrentEraValidators<T>>::put(validators);
    }

    fn set_candidate_validators(candidate_validators: Vec<T::AccountId>) {
        <CandidateValidators<T>>::put(candidate_validators);
    }

    fn delegated_score_of_validator(validator: &T::AccountId) -> Option<u64> {
        let era_index = <CurrentEra>::get().unwrap_or(0);
        if <Delegators<T>>::contains_key(era_index, validator.clone()) {
            let delegators = <Delegators<T>>::get(era_index, validator);
            let mut score: u64 = 0;
            for (_, s) in delegators {
                score += s;
            }
            Some(score)
        } else {
            Some(0)
        }
    }

    fn total_delegated_score() -> Option<u64> {
        let mut total_score: u64 = 0;
        if let Some(candidate_validators) = <CandidateValidators<T>>::get() {
            for candidate_validator in candidate_validators {
                total_score = total_score.saturating_add(
                    Self::delegated_score_of_validator(&candidate_validator).unwrap_or(0),
                );
            }
        }
        Some(total_score)
    }

    // 获取账号 account id 质押的 score
    fn get_delegated_score(account_id: T::AccountId) -> Option<u64> {
        if CreditLedger::<T>::contains_key(account_id.clone()) {
            let ledger = CreditLedger::<T>::get(account_id);
            let score = ledger.delegated_score;
            Some(score)
        } else {
            Some(0)
        }
    }

    fn kill_credit(account_id: T::AccountId) -> bool {
        if CreditLedger::<T>::contains_key(account_id.clone()) {
            let ledger = <CreditLedger<T>>::get(account_id.clone());
            let validator = ledger.validator_account;

            let current_era = Self::current_era().unwrap_or(0);

            let mut start_index = 0;
            if current_era > 84 {
                start_index = current_era - 84;
            }

            for era in start_index..current_era + 1 {
                if <Delegators<T>>::contains_key(era, validator.clone()) {
                    let delegators = <Delegators<T>>::take(era, validator.clone());
                    let next_delegators: Vec<_> = delegators
                        .iter()
                        .filter(|(delegator, _)| delegator != &account_id)
                        .collect();
                    <Delegators<T>>::insert(era, validator.clone(), next_delegators)
                }
            }
            true
        } else {
            false
        }
    }

    fn get_total_validator_score(era_index: EraIndex, validator: T::AccountId) -> Option<u64> {
        if <Delegators<T>>::contains_key(era_index, validator.clone()) {
            let delegators = <Delegators<T>>::get(era_index, validator);
            let mut total_score: u64 = 0;
            for (_, s) in delegators {
                total_score += s;
            }
            Some(total_score)
        } else {
            Some(0)
        }
    }

    fn set_eras_reward(era_index: EraIndex, total_reward:BalanceOf<T>) {
        <ErasValidatorReward<T>>::insert(era_index, total_reward);
    }

    fn payout_delegators(era_index: EraIndex, commission: Perbill, validator: T::AccountId,
                         validator_payee: T::AccountId)  -> bool {
        if !<ErasValidatorReward<T>>::contains_key(&era_index) {
            return false;
        }
        let era_payout = <ErasValidatorReward<T>>::get(&era_index).unwrap();

        let validator_commission_payout = commission * era_payout;
        let validator_leftover_payout = era_payout - validator_commission_payout;

        // We can now make total validator payout:
        if let Some(imbalance) = Self::make_payout(
            validator_payee.clone(),
            validator_commission_payout,
         ) {
             Self::deposit_event(RawEvent::Reward(validator_payee, imbalance.peek()));
        }

        // Lets now calculate how this is split to the nominators.
        // Reward only the clipped exposures. Note this is not necessarily sorted.
        let era_total_score =
            Self::get_total_validator_score(era_index,validator.clone()).unwrap();
        let delegators = <Delegators<T>>::get(era_index, validator.clone());
        for (who, s) in delegators {
            let delegator_exposure_part =
                Perbill::from_rational_approximation(s, era_total_score);

            let delegator_reward: BalanceOf<T> =
                delegator_exposure_part * validator_leftover_payout;
            // We can now make nominator payout:
            if let Some(imbalance) = Self::make_payout(who.clone(), delegator_reward) {
                Self::deposit_event(RawEvent::Reward(who, imbalance.peek()));
            }
        }
        true
    }

    fn make_payout(receiver: T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
        Some(T::Currency::deposit_creating(&receiver, amount))
    }
}
