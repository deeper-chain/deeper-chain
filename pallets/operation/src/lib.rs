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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;
use scale_info::TypeInfo;
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use codec::{Decode, Encode, MaxEncodedLen};
    use frame_support::traits::{
        Currency, ExistenceRequirement, Get, Imbalance, LockIdentifier, LockableCurrency,
        OnUnbalanced, ReservableCurrency, WithdrawReasons,
    };
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*, transactional,
    };
    use frame_system::pallet_prelude::*;
    use frame_system::{self, ensure_signed};
    use node_primitives::{
        credit::CreditInterface,
        user_privileges::{Privilege, UserPrivilegeInterface},
        OperationInterface, DPR,
    };
    use scale_info::prelude::string::{String, ToString};
    pub use sp_core::H160;
    use sp_runtime::{
        traits::{Saturating, StaticLookup, UniqueSaturatedFrom, UniqueSaturatedInto, Zero},
        Percent, RuntimeDebug,
    };

    type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub type NegativeImbalanceOf<T> = <<T as pallet::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    pub const MILLISECS_PER_DAY: u64 = 1000 * 3600 * 24;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type MaxMember: Get<u32>;
        type OPWeightInfo: WeightInfo;
        type BurnedTo: OnUnbalanced<NegativeImbalanceOf<Self>>;
        type MinimumBurnedDPR: Get<BalanceOf<Self>>;
        type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;
        type UserPrivilegeInterface: UserPrivilegeInterface<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Locked(T::AccountId, BalanceOf<T>),
        UnLocked(T::AccountId),
        Unreserve(T::AccountId, BalanceOf<T>),
        ReleaseReward(T::AccountId, BalanceOf<T>),
        AccountReleaseEnd(T::AccountId),
        SingleReleaseTooMuch(T::AccountId, BalanceOf<T>),
        BurnForEZC(T::AccountId, BalanceOf<T>, H160),
        UnstakingResult(T::AccountId, String),
        GetNpowReward(T::AccountId, H160),
        NpowMint(T::AccountId, BalanceOf<T>),
        BridgeDeeperToOther(H160, T::AccountId, BalanceOf<T>, String),
        BridgeOtherToDeeper(T::AccountId, H160, BalanceOf<T>, String),
        DPRPrice(BalanceOf<T>, H160),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NotMatchOwner,
        ReachDailyMaximumLimit,
        ReachSingleMaximumLimit,
        ReleaseDayZero,
        BurnedDprTooLow,
        ReleaseBalanceZero,
        UnauthorizedAccounts,
        NpowRewardAddressNotFound,
        FundPoolNotSet,
        PriceDiffTooMuch,
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum Releases {
        V1_0_0,
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct ReleaseInfo<T: Config> {
        account: T::AccountId,
        total_release_days: u32,
        start_release_moment: u64,
        total_balance: BalanceOf<T>,
    }

    impl<T: Config> ReleaseInfo<T> {
        pub fn new(
            account: T::AccountId,
            total_release_days: u32,
            start_release_moment: u64,
            total_balance: BalanceOf<T>,
        ) -> Self {
            Self {
                account,
                total_release_days,
                start_release_moment,
                total_balance,
            }
        }
    }

    impl<T: Config> sp_std::fmt::Debug for ReleaseInfo<T> {
        fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
            write!(
                f,
                "account {:?} days {} start {}, balance {:?}",
                self.account,
                self.total_release_days,
                self.start_release_moment,
                self.total_balance
            )
        }
    }

    #[derive(Encode, Decode, Clone, Debug, MaxEncodedLen, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct CurrentRelease<T: Config> {
        pub basic_info: ReleaseInfo<T>,
        pub start_day: u32,
        pub last_release_day: u32,
        pub balance_per_day: BalanceOf<T>,
    }

    #[pallet::storage]
    #[pallet::getter(fn total_release)]
    pub(super) type TotalRelease<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultDailyMax<T: Config>() -> BalanceOf<T> {
        UniqueSaturatedFrom::unique_saturated_from(1_000_000 * DPR)
    }

    #[pallet::storage]
    #[pallet::getter(fn daily_max_limit)]
    pub(super) type DailyMaxLimit<T: Config> =
        StorageValue<_, BalanceOf<T>, ValueQuery, DefaultDailyMax<T>>;

    #[pallet::type_value]
    pub fn DefaultSingleMax<T: Config>() -> BalanceOf<T> {
        UniqueSaturatedFrom::unique_saturated_from(10_000 * DPR)
    }

    #[pallet::storage]
    #[pallet::getter(fn single_max_limit)]
    pub(super) type SingleMaxLimit<T: Config> =
        StorageValue<_, BalanceOf<T>, ValueQuery, DefaultSingleMax<T>>;

    #[pallet::storage]
    #[pallet::getter(fn total_daily_release)]
    pub type TotalDailyRelease<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn accounts_release_info)]
    pub type AccountsReleaseInfo<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, CurrentRelease<T>, OptionQuery>;

    /// delegators last key
    #[pallet::storage]
    #[pallet::getter(fn account_release_last_key)]
    pub(crate) type AccountReleaseLastKey<T> = StorageValue<_, Vec<u8>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn saved_day)]
    pub(crate) type SavedDay<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn day_release_end)]
    pub(crate) type DayReleaseEnd<T> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_fund_address)]
    pub(crate) type BridgeFundAddreess<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::storage]
    pub(super) type StorageVersion<T> = StorageValue<_, Releases>;

    #[pallet::storage]
    #[pallet::getter(fn dpr_price)]
    pub(super) type DprPrice<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn price_diff_rate)]
    pub(super) type PriceDiffRate<T: Config> = StorageValue<_, Percent, OptionQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_: T::BlockNumber) {
            let saved_day = Self::saved_day();
            let cur_time: u64 = <pallet_timestamp::Pallet<T>>::get().unique_saturated_into();
            let cur_day = (cur_time / MILLISECS_PER_DAY) as u32;
            if cur_day > saved_day {
                SavedDay::<T>::put(cur_day);
                DayReleaseEnd::<T>::put(false);
                let prefix = Self::get_account_release_prefix_hash();
                AccountReleaseLastKey::<T>::put(prefix);
            } else {
                Self::release_staking_balance(cur_day);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::OPWeightInfo::force_remove_lock())]
        pub fn force_remove_lock(
            origin: OriginFor<T>,
            id: LockIdentifier,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;
            <T::Currency as LockableCurrency<_>>::remove_lock(id, &who);
            Self::deposit_event(Event::UnLocked(who));
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::force_reserve_by_member())]
        pub fn force_reserve_by_member(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Self::is_locker_member(&sender),
                Error::<T>::UnauthorizedAccounts
            );
            let who = T::Lookup::lookup(who)?;
            <T::Currency as ReservableCurrency<_>>::reserve(&who, value)?;
            Self::deposit_event(Event::Locked(sender, value));
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::set_release_limit_parameter())]
        pub fn set_release_limit_parameter(
            origin: OriginFor<T>,
            #[pallet::compact] single_max_limit: BalanceOf<T>,
            #[pallet::compact] daily_max_limit: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            <SingleMaxLimit<T>>::put(single_max_limit);
            <DailyMaxLimit<T>>::put(daily_max_limit);

            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::unstaking_release())]
        #[transactional]
        pub fn unstaking_release(
            origin: OriginFor<T>,
            basic_info: ReleaseInfo<T>,
        ) -> DispatchResult {
            let setter = ensure_signed(origin)?;

            if !Self::is_payment_address(&setter) {
                Self::deposit_event(Event::UnstakingResult(
                    setter,
                    "not release owner".to_string(),
                ));
                return Err(Error::<T>::NotMatchOwner.into());
            }

            let account = basic_info.account.clone();

            let remainder_release_days = basic_info.total_release_days;
            if remainder_release_days == 0 {
                Self::deposit_event(Event::UnstakingResult(
                    account,
                    "release day is zero".to_string(),
                ));
                return Err(Error::<T>::ReleaseDayZero.into());
            }

            if basic_info.total_balance.is_zero() {
                Self::deposit_event(Event::UnstakingResult(
                    account,
                    "release balance is zero".to_string(),
                ));
                return Err(Error::<T>::ReleaseBalanceZero.into());
            }

            let start_day = (basic_info.start_release_moment / MILLISECS_PER_DAY) as u32;
            let balance_per_day = basic_info.total_balance / remainder_release_days.into();
            let single_max_limit = Self::single_max_limit();
            if balance_per_day > single_max_limit {
                Self::deposit_event(Event::UnstakingResult(
                    account,
                    "more than single max limit".to_string(),
                ));
                return Err(Error::<T>::ReachSingleMaximumLimit.into());
            }

            T::CreditInterface::do_unstaking_slash_credit(&account)?;
            let cur_info = CurrentRelease::<T> {
                basic_info,
                last_release_day: start_day,
                start_day,
                balance_per_day,
            };
            AccountsReleaseInfo::<T>::insert(&account, cur_info);
            Self::deposit_event(Event::UnstakingResult(account, "success".to_string()));
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::unstaking_release())]
        pub fn remove_staking_release_info(
            origin: OriginFor<T>,
            release_accounts: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let setter = ensure_signed(origin)?;
            ensure!(Self::is_payment_address(&setter), Error::<T>::NotMatchOwner);
            for account in release_accounts {
                AccountsReleaseInfo::<T>::remove(&account);
            }
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::burn_for_ezc())]
        pub fn burn_for_ezc(
            origin: OriginFor<T>,
            burned: BalanceOf<T>,
            benifity: H160,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                burned >= T::MinimumBurnedDPR::get(),
                Error::<T>::BurnedDprTooLow
            );
            let burned = T::Currency::withdraw(
                &sender,
                burned,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::KeepAlive,
            )?;
            let balance = burned.peek();
            T::BurnedTo::on_unbalanced(burned);
            T::CreditInterface::burn_record(balance);
            Self::deposit_event(Event::<T>::BurnForEZC(sender, balance, benifity));
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::npow_mint())]
        pub fn npow_mint(
            origin: OriginFor<T>,
            target: T::AccountId,
            dpr: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::NpowMint),
                Error::<T>::UnauthorizedAccounts
            );
            T::Currency::deposit_creating(&target, dpr);
            Self::deposit_event(Event::<T>::NpowMint(target, dpr));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_fund_pool_address(
            origin: OriginFor<T>,
            funder: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::BridgeAdmin),
                Error::<T>::UnauthorizedAccounts
            );
            BridgeFundAddreess::<T>::put(funder);
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::bridge_deeper_to_other())]
        pub fn bridge_deeper_to_other(
            origin: OriginFor<T>,
            to: H160,
            from: T::AccountId,
            amount: BalanceOf<T>,
            tx: String,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::BridgeAdmin),
                Error::<T>::UnauthorizedAccounts
            );
            let funder = Self::bridge_fund_address();
            ensure!(funder.is_some(), Error::<T>::FundPoolNotSet);

            T::Currency::transfer(
                &from,
                &funder.unwrap(),
                amount,
                ExistenceRequirement::KeepAlive,
            )?;
            Self::deposit_event(Event::<T>::BridgeDeeperToOther(to, from, amount, tx));
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::bridge_other_to_deeper())]
        pub fn bridge_other_to_deeper(
            origin: OriginFor<T>,
            to: T::AccountId,
            from: H160,
            amount: BalanceOf<T>,
            tx: String,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::BridgeAdmin),
                Error::<T>::UnauthorizedAccounts
            );
            let funder = Self::bridge_fund_address();
            ensure!(funder.is_some(), Error::<T>::FundPoolNotSet);

            T::Currency::transfer(
                &funder.unwrap(),
                &to,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;
            Self::deposit_event(Event::<T>::BridgeOtherToDeeper(to, from, amount, tx));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_price_diff_rate(
            origin: OriginFor<T>,
            price_diff_rate: Percent,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::UnauthorizedAccounts
            );
            PriceDiffRate::<T>::put(price_diff_rate);
            Ok(().into())
        }

        #[pallet::weight(T::OPWeightInfo::set_dpr_price())]
        pub fn set_dpr_price(
            origin: OriginFor<T>,
            price: BalanceOf<T>,
            worker: H160,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::UnauthorizedAccounts
            );

            let rate = Self::price_diff_rate();
            let old_price = Self::dpr_price();

            match (rate, old_price) {
                (Some(rate), Some(old_price)) => {
                    let diff_limit = rate * old_price;
                    let diff = {
                        if price > old_price {
                            price - old_price
                        } else {
                            old_price - price
                        }
                    };
                    ensure!(diff <= diff_limit, Error::<T>::PriceDiffTooMuch);
                }
                _ => {}
            }

            DprPrice::<T>::put(price);
            Self::deposit_event(Event::<T>::DPRPrice(price, worker));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn get_account_release_prefix_hash() -> Vec<u8> {
            use frame_support::storage::generator::StorageMap;
            AccountsReleaseInfo::<T>::prefix_hash()
        }

        fn next_account_release_key(last_key: &Vec<u8>) -> Vec<u8> {
            sp_io::storage::next_key(last_key).unwrap_or(Vec::<u8>::new())
        }

        fn get_account_release_data(next_key: &Vec<u8>) -> Option<CurrentRelease<T>> {
            frame_support::storage::unhashed::get::<CurrentRelease<T>>(next_key)
        }

        fn release_staking_balance(cur_day: u32) {
            if Self::day_release_end() {
                return;
            }
            let prefix = Self::get_account_release_prefix_hash();
            let last_key = Self::account_release_last_key().unwrap_or(prefix.clone());
            let mut next_key = Self::next_account_release_key(&last_key);
            let mut total_release = Self::total_release();
            let mut daily_release = Self::total_daily_release(cur_day);
            let mut to_be_removed = Vec::new();
            let mut blockly_release = BalanceOf::<T>::zero();

            loop {
                if next_key.starts_with(&prefix) {
                    let data = Self::get_account_release_data(&next_key);
                    if data.is_none() {
                        break;
                    }
                    let data = data.unwrap();

                    let released_balance = data.balance_per_day
                        * (cur_day.saturating_sub(data.last_release_day).into());

                    // It happens when setting release start day greater than current day
                    if released_balance.is_zero() {
                        next_key = Self::next_account_release_key(&next_key);
                        continue;
                    }
                    if released_balance > Self::single_max_limit() {
                        Self::deposit_event(Event::<T>::SingleReleaseTooMuch(
                            data.basic_info.account,
                            released_balance,
                        ));
                        break;
                    }

                    let imbalance =
                        T::Currency::deposit_creating(&data.basic_info.account, released_balance);
                    total_release += released_balance;
                    daily_release += released_balance;
                    blockly_release += released_balance;

                    AccountsReleaseInfo::<T>::mutate(data.basic_info.account.clone(), |info| {
                        info.as_mut().unwrap().last_release_day = cur_day;
                    });

                    if cur_day.saturating_sub(data.start_day) >= data.basic_info.total_release_days
                    {
                        to_be_removed.push(data.basic_info.account.clone());
                    }

                    Self::deposit_event(Event::<T>::ReleaseReward(
                        data.basic_info.account,
                        imbalance.peek(),
                    ));
                    if daily_release >= Self::daily_max_limit() {
                        DayReleaseEnd::<T>::put(true);
                        break;
                    }
                    if blockly_release >= Self::single_max_limit() {
                        // One block only release single_max_limit DPR
                        break;
                    }
                    next_key = Self::next_account_release_key(&next_key);
                } else {
                    DayReleaseEnd::<T>::put(true);
                    break;
                }
            }
            TotalDailyRelease::<T>::insert(cur_day, daily_release);
            TotalRelease::<T>::put(total_release);
            AccountReleaseLastKey::<T>::put(next_key);
            for account in to_be_removed {
                AccountsReleaseInfo::<T>::remove(account);
            }
        }

        fn is_locker_member(user: &T::AccountId) -> bool {
            T::UserPrivilegeInterface::has_privilege(user, Privilege::LockerMember)
        }

        fn is_payment_address(user: &T::AccountId) -> bool {
            T::UserPrivilegeInterface::has_privilege(user, Privilege::ReleaseSetter)
        }
    }

    impl<T: Config> OperationInterface<T::AccountId, BalanceOf<T>> for Pallet<T> {
        fn is_single_max_limit(pay_amount: BalanceOf<T>) -> bool {
            if Self::single_max_limit() >= pay_amount {
                let cur_daily_release = Self::total_daily_release(Self::saved_day());
                Self::daily_max_limit() >= cur_daily_release.saturating_add(pay_amount)
            } else {
                false
            }
        }
    }
}
