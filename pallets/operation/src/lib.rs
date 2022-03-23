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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use codec::{Decode, Encode, MaxEncodedLen};
    use frame_support::traits::{
        Currency, Get, LockIdentifier, LockableCurrency, ReservableCurrency, WithdrawReasons,
    };
    use frame_support::WeakBoundedVec;
    use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_system::{self, ensure_signed};
    use sp_runtime::{traits::StaticLookup, RuntimeDebug};

    type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
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
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// not in locker members
        NotLockMember,
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum Releases {
        V1_0_0,
    }

    #[pallet::storage]
    #[pallet::getter(fn lock_member_whitelist)]
    pub(super) type LockMemberWhiteList<T: Config> =
        StorageValue<_, WeakBoundedVec<T::AccountId, T::MaxMember>, ValueQuery>;

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
            } else {
                StorageVersion::<T>::put(Releases::V1_0_0);
            }
            T::DbWeight::get().reads_writes(1, 1)
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
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

        #[pallet::weight(T::WeightInfo::force_unreserve())]
        pub fn force_unreserve(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;
            <T::Currency as ReservableCurrency<_>>::unreserve(&who, value);
            Self::deposit_event(Event::Unreserve(who, value));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_lock_members())]
        pub fn set_lock_members(
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

        #[pallet::weight(T::WeightInfo::force_lock())]
        pub fn force_lock(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            #[pallet::compact] value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            const FORCE_LOCK_ID: [u8; 8] = *b"forcelck";
            let sender = ensure_signed(origin)?;
            ensure!(
                <LockMemberWhiteList<T>>::get().contains(&sender),
                Error::<T>::NotLockMember
            );
            let who = T::Lookup::lookup(who)?;
            <T::Currency as LockableCurrency<_>>::set_lock(
                FORCE_LOCK_ID,
                &who,
                value,
                WithdrawReasons::all(),
            );
            Self::deposit_event(Event::Locked(sender, value));
            Ok(().into())
        }
    }
}
