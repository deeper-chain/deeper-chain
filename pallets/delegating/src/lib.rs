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
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Currency,
};
use frame_system::ensure_signed;
use pallet_credit::CreditInterface;
use sp_std::vec;
use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait{
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type CreditInterface: CreditInterface<Self::AccountId>;
}

pub type EraIndex = u32;

#[derive(Default, Clone, Encode, Decode)]
pub struct CreditScoreLedger<AccountId> {
    delegated_account: AccountId,
    delegated_score: u64,
    validator_account: AccountId,
    withdraw_era: EraIndex,
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Trait> as Delegating  {
        Something get(fn something): Option<u32>;

        //DelegatedScore get(fn delegated): map hasher(blake2_128_concat) T::AccountId  => u64;

        // 存质押的credit 及 delegater id
        CreditLedger get(fn credit_ledger): map hasher(blake2_128_concat) T::AccountId
            => CreditScoreLedger<T::AccountId>;

        // 存PoC生效的Era, validator id
        Delegators get(fn delegators): double_map hasher(blake2_128_concat) EraIndex,
        hasher(blake2_128_concat) T::AccountId => Vec<(T::AccountId, u64)>;

        //	TODO should be update when era change
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
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        Delegated(AccountId, AccountId, u64),
        UnDelegated(AccountId),
        WithdrawCredit(AccountId, u64),
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
        NoCreditLedgerData,
        NotRightEra,
        CreditScoreTooLow,
        NonCandidateValidator,
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

        ///
        /// TODO score 参数不需要， 实际 信誉分 通过 credit pallet 模块获取
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
                        let current_era = CurrentEra::get().unwrap_or(0);
                        let last_validator = _ledger.validator_account;

                        // remove controller from old validator
                        if Delegators::<T>::contains_key(current_era, last_validator.clone()){
                            let delegators = Delegators::<T>::take(current_era, last_validator.clone());
                            let next_delegators: Vec<_> = delegators
                                .iter()
                                .filter(|(delegator, _)|{
                                    delegator != &controller
                                })
                                .collect();
                            Delegators::<T>::insert(current_era, last_validator.clone(), next_delegators);
                        }

                        // add controller to new validator
                        if Delegators::<T>::contains_key(current_era, validator.clone()){
                            let mut delegators = Delegators::<T>::take(current_era, validator.clone());
                            delegators.push((controller.clone(), score));
                            Delegators::<T>::insert(current_era, validator.clone(), delegators);
                        }else{
                            let delegators = vec![(controller.clone(),score)];
                            Delegators::<T>::insert(current_era, validator.clone(), delegators);
                        }

                    }else{ // target is same, update score
                        CreditLedger::<T>::mutate(controller.clone(), |ledger| {
                            (*ledger).validator_account = validator.clone();
                            (*ledger).delegated_score = score;
                            (*ledger).withdraw_era = 0;
                        });
                        let current_era = CurrentEra::get().unwrap_or(0);

                        if Delegators::<T>::contains_key(current_era, validator.clone()){
                            let delegators = Delegators::<T>::take(current_era, validator.clone());
                            let next_delegators: Vec<_> = delegators
                                .iter()
                                .map(|(delegator, s)|{
                                    if delegator == &controller{
                                        (delegator, score)
                                    }else{
                                        (delegator, *s)
                                    }
                                })
                                .collect();
                            Delegators::<T>::insert(current_era, validator.clone(), next_delegators);
                        }

                    }
                }else{ // delegate
                    let ledger = CreditScoreLedger{
                        delegated_account: controller.clone(),
                        delegated_score: score,
                        validator_account: validator.clone(),
                        withdraw_era: 0,
                    };
                    CreditLedger::<T>::insert(controller.clone(), ledger.clone());

                    let current_era = CurrentEra::get().unwrap_or(0);
                    if Delegators::<T>::contains_key(current_era, validator.clone()){
                        let mut delegators = Delegators::<T>::take(current_era, validator.clone());
                        delegators.push((controller.clone(),score));
                        Delegators::<T>::insert(current_era, validator.clone(), delegators);
                    }else{
                        let delegators = vec![(controller.clone(),score)];
                        Delegators::<T>::insert(current_era, validator.clone(), delegators);
                    }

                    Self::deposit_event(RawEvent::Delegated(controller, validator, score));
                }
            }
            Ok(())
        }

        #[weight = 10_000]
        pub fn undelegate(origin) -> dispatch::DispatchResult {
            info!("[FLQ] will undelegate credit score ");

            // 合法性检查
            let controller = ensure_signed(origin)?;
            ensure!(CreditLedger::<T>::contains_key(controller.clone()), Error::<T>::NotDelegate);

            let current_era = CurrentEra::get().unwrap_or(0);
            // TODO 赎回锁定周期为 当前ERA_index + 信誉分锁定周期数（不要使用固定值 2）
            info!("[FLQ] current era index : {:?}", current_era);
            let withdraw_era = current_era + 2;

            // 更新 withdraw_era 字段
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
            info!("[FLQ] withdraw credit score");

            let controller = ensure_signed(origin)?;

            // 检查该账户是否存在 待赎回的 credit score
            if !CreditLedger::<T>::contains_key(controller.clone()) {
                error!("[FLQ] can't found credit ledger for your account ");
                Err(Error::<T>::NoCreditLedgerData)?
            }

            // 检查 credit score 是否达到可赎回的条件（是否过了锁定期）
            let ledger = CreditLedger::<T>::get(controller.clone());
            let current_era = CurrentEra::get().unwrap_or(0);
            if ledger.withdraw_era > current_era {
                error!("[FLQ] can't withdraw credit score because it's not the right time yet ");
                Err(Error::<T>::NotRightEra)?
            }

            // 删除该账户对应的 CreditLedger
            CreditLedger::<T>::remove(controller.clone());

            Self::deposit_event(RawEvent::WithdrawCredit(controller, ledger.delegated_score));

            Ok(())
        }

        ///	用户重新委托信誉分（针对： 已经发起 undelegate，信誉分还处于锁定状态的情况）
        #[weight = 10_000]
        pub fn redelegate(origin) -> dispatch::DispatchResult {
            info!("[FLQ] redelegate credit score ");
            let controller = ensure_signed(origin)?;
            ensure!(CreditLedger::<T>::contains_key(controller.clone()), Error::<T>::NotDelegate);

            CreditLedger::<T>::mutate(controller.clone(),|ledger| (*ledger).withdraw_era = 0);

            Ok(())
        }

        #[weight = 10_000]
        pub fn getdelegators(origin, era_index: u32, validator: T::AccountId) -> dispatch::DispatchResult{
            info!("[FLQ] get delegators for era_index : {:?}", era_index);
            // 查看指定 era 周期内对应的 delegators
            // TODO 待实现

            Ok(())
        }
    }
}

pub trait CreditDelegateInterface<AccountId> {
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
}
//定义公共和私有函数

impl<T: Trait> CreditDelegateInterface<T::AccountId> for Module<T> {
    /// 每个era开始调用一次
    fn set_current_era(current_era: EraIndex) {
        <CurrentEra>::put(current_era);

        if current_era > 0 {
            // 更新潜在validator背后质押credit的账户、及era_index
            if let Some(candidate_validators) = <CandidateValidators<T>>::get() {
                for candidate_validator in candidate_validators {
                    let delegators =
                        <Delegators<T>>::get(current_era - 1, candidate_validator.clone());
                    let next_delegators: Vec<_> = delegators
                        .iter()
                        .filter(|(delegator, _)| {
                            let ledger = <CreditLedger<T>>::get(delegator);
                            ledger.withdraw_era == 0 // ==0 正常质押credit
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
        if <Delegators<T>>::contains_key(era_index, validator.clone()){
            let delegators = <Delegators<T>>::get(era_index, validator);
            let mut score: u64 = 0;
            for (_, s) in delegators {
                score += s;
            }
            Some(score)
        }else{
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
        }
        false
    }
}
