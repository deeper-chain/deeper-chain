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

mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

pub mod weights;
use scale_info::TypeInfo;
use sp_std::prelude::*;
pub use weights::WeightInfo;

pub type EraIndex = u32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use codec::{Decode, Encode, MaxEncodedLen};
    use frame_support::traits::{
        Currency, Get, Imbalance, LockIdentifier, LockableCurrency, ReservableCurrency,
    };
    use frame_support::WeakBoundedVec;
    use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_system::{self, ensure_signed};
    use pallet_credit::EraIndex;
    use sp_runtime::{traits::Saturating, traits::StaticLookup, RuntimeDebug};

    type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        type MaxMember: Get<u32>;
        type WeightInfo: WeightInfo;
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
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// not in locker members
        NotLockMember,
        NotReleaseOwnerAddress,
        NotMatchOwner,
        ReachDailyMaximumLimit,
        ReachSingleMaximumLimit,
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum Releases {
        V1_0_0,
    }

    #[pallet::storage]
    #[pallet::getter(fn total_release)]
    pub(super) type TotalRelease<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn daily_max_limit)]
    pub(super) type DailyMaxLimit<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn single_max_limit)]
    pub(super) type SingleMaxLimit<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn lock_member_whitelist)]
    pub(super) type LockMemberWhiteList<T: Config> =
        StorageValue<_, WeakBoundedVec<T::AccountId, T::MaxMember>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn release_payment_address)]
    pub type ReleasePaymentAddress<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_daily_release)]
    pub type TotalDailyRelease<T: Config> =
        StorageMap<_, Blake2_128Concat, EraIndex, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            if StorageVersion::<T>::get().is_none() {
                frame_support::storage::migration::move_storage_from_pallet(
                    b"LockMemberWhiteList",
                    b"Balances",
                    b"Operation",
                );
                StorageVersion::<T>::put(Releases::V1_0_0);
                return T::DbWeight::get().reads_writes(1, 1);
            }
            0
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::force_remove_lock())]
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

        #[pallet::weight(T::WeightInfo::set_reserve_members())]
        pub fn set_reserve_members(
            origin: OriginFor<T>,
            whitelist: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let whitelist = WeakBoundedVec::<_, T::MaxMember>::force_from(
                whitelist,
                Some("Balances Update lock function whitelist"),
            );

            if whitelist.len() as u32 > T::MaxMember::get() {
                log::warn!("Whitelist too large.");
            }
            <LockMemberWhiteList<T>>::put(whitelist);
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::force_reserve_by_member())]
        pub fn force_reserve_by_member(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                <LockMemberWhiteList<T>>::get().contains(&sender),
                Error::<T>::NotLockMember
            );
            let who = T::Lookup::lookup(who)?;
            <T::Currency as ReservableCurrency<_>>::reserve(&who, value)?;
            Self::deposit_event(Event::Locked(sender, value));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_release_owner_address())]
        pub fn set_release_owner_address(
            origin: OriginFor<T>,
            owner: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            <ReleasePaymentAddress<T>>::put(owner.clone());
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_release_limit_parameter())]
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

        #[pallet::weight(T::WeightInfo::staking_release())]
        pub fn staking_release(
            origin: OriginFor<T>,
            who: T::AccountId,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            let owner =
                Self::release_payment_address().ok_or(Error::<T>::NotReleaseOwnerAddress)?;

            ensure!(admin == owner, Error::<T>::NotMatchOwner);
            ensure!(
                value <= Self::single_max_limit(),
                Error::<T>::ReachSingleMaximumLimit
            );

            let current_era = Self::get_current_era();
            let daily_release_total = value.saturating_add(Self::total_daily_release(current_era));

            ensure!(
                daily_release_total <= Self::daily_max_limit(),
                Error::<T>::ReachDailyMaximumLimit
            );

            let imbalance = T::Currency::deposit_creating(&who, value);
            TotalDailyRelease::<T>::insert(current_era, daily_release_total);
            let total = value.saturating_add(Self::total_release());
            <TotalRelease<T>>::put(total);

            Self::deposit_event(Event::<T>::ReleaseReward(who.clone(), imbalance.peek()));

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn get_current_era() -> EraIndex {
            let block_number = <frame_system::Pallet<T>>::block_number();
            TryInto::<EraIndex>::try_into(block_number / T::BlocksPerEra::get())
                .ok()
                .unwrap()
        }
    }
}
