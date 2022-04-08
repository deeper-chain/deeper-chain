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
        Currency, Get, LockIdentifier, LockableCurrency, ReservableCurrency,
    };
    use frame_support::WeakBoundedVec;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*, transactional,
    };
    use frame_system::pallet_prelude::*;
    use frame_system::{self, ensure_signed};
    use pallet_credit::CreditInterface;
    use sp_runtime::{traits::StaticLookup, RuntimeDebug};

    type BalanceOf<T> = <<T as pallet::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    pub type ClassIdOf<T> = <T as pallet_uniques::Config>::ClassId;
    pub type InstanceIdOf<T> = <T as pallet_uniques::Config>::InstanceId;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_uniques::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type MaxMember: Get<u32>;
        // CreditInterface of credit pallet
        type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;
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
        UpdateNftCredit(ClassIdOf<T>, u64),
        BurnNft(T::AccountId, ClassIdOf<T>, InstanceIdOf<T>, u64),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// not in locker members
        NotLockMember,
        MiningMachineClassCreditNoConfig,
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

    #[pallet::storage]
    #[pallet::getter(fn tips)]
    pub type MiningMachineClassCredit<T: Config> =
        StorageMap<_, Twox64Concat, ClassIdOf<T>, u64, ValueQuery>;

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
        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_remove_lock())]
        pub fn force_remove_lock(
            origin: OriginFor<T>,
            id: LockIdentifier,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let who = T::Lookup::lookup(who)?;
            <<T as pallet::Config>::Currency as LockableCurrency<_>>::remove_lock(id, &who);
            Self::deposit_event(Event::UnLocked(who));
            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_reserve_members())]
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

        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_reserve_by_member())]
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
            <<T as pallet::Config>::Currency as ReservableCurrency<_>>::reserve(&who, value)?;
            Self::deposit_event(Event::Locked(sender, value));
            Ok(().into())
        }

        //#[pallet::weight(<T as pallet::Config>::WeightInfo::force_reserve_by_member())]
        #[pallet::weight(0)]
        pub fn update_nft_class_credit(
            origin: OriginFor<T>,
            #[pallet::compact] class_id: ClassIdOf<T>,
            credit: u64,
        ) -> DispatchResultWithPostInfo {
            let _sender = ensure_root(origin)?;

            MiningMachineClassCredit::<T>::insert(class_id, credit);

            Self::deposit_event(Event::UpdateNftCredit(class_id, credit));
            Ok(().into())
        }

        //#[pallet::weight(<T as pallet::Config>::WeightInfo::brun_nft())]
        #[pallet::weight(0)]
        #[transactional]
        pub fn brun_nft(
            origin: OriginFor<T>,
            #[pallet::compact] class_id: ClassIdOf<T>,
            #[pallet::compact] instance_id: InstanceIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;

            ensure!(
                MiningMachineClassCredit::<T>::contains_key(&class_id),
                Error::<T>::MiningMachineClassCreditNoConfig
            );

            pallet_uniques::Pallet::<T>::burn(origin, class_id, instance_id, None)?;

            let credit = MiningMachineClassCredit::<T>::get(&class_id);
            T::CreditInterface::update_credit_by_burn_nft(sender.clone(), credit)?;

            Self::deposit_event(Event::BurnNft(sender, class_id, instance_id, credit));
            Ok(().into())
        }
    }
}
