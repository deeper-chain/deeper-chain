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

// #[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;
mod weights;

pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::DispatchResult;
    use frame_support::traits::{
        fungibles::metadata::Mutate as MetaMutate, fungibles::Create, fungibles::Inspect,
        fungibles::Mutate, nonfungibles::Mutate as NftMutate, Time,
    };
    use frame_support::{
        dispatch::RawOrigin, pallet_prelude::*, transactional, weights::Weight, PalletId,
    };
    use frame_system::pallet_prelude::*;
    use node_primitives::{
        user_privileges::{Privilege, UserPrivilegeInterface},
        DPR,
    };

    use sp_core::H160;
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, UniqueSaturatedFrom, UniqueSaturatedInto},
        Perbill,
    };
    use sp_std::{convert::TryInto, prelude::*};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_uniques::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Currency
        type AdscCurrency: MetaMutate<Self::AccountId>
            + Mutate<Self::AccountId>
            + Create<Self::AccountId>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
        /// query user prvileges
        type UserPrivilegeInterface: UserPrivilegeInterface<Self::AccountId>;

        type Time: Time;

        #[pallet::constant]
        type AdscId: Get<AssetIdOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    pub(crate) type AssetIdOf<T> =
        <<T as Config>::AdscCurrency as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

    pub(crate) type AssetBalanceOf<T> =
        <<T as Config>::AdscCurrency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ClassIdOf<T> = <T as pallet_uniques::Config>::CollectionId;
    pub type InstanceIdOf<T> = <T as pallet_uniques::Config>::ItemId;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn adsc_stakers)]
    pub type AdscStakers<T: Config> =
        CountedStorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn adsc_nfts)]
    pub type AdscNfts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (ClassIdOf<T>, InstanceIdOf<T>), OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn adsc_staker_last_key)]
    pub(crate) type AdscStakerLastKey<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::storage]
    pub type CurrentAdscBaseReward<T: Config> =
        StorageValue<_, AssetBalanceOf<T>, ValueQuery, AdscInitReward<T>>;

    #[pallet::type_value]
    pub fn AdscInitReward<T: Config>() -> AssetBalanceOf<T> {
        UniqueSaturatedFrom::unique_saturated_from(8235 * DPR)
    }

    #[pallet::storage]
    pub type CurrentMintedAdsc<T: Config> = StorageValue<_, AssetBalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type CurrentHalfTarget<T: Config> =
        StorageValue<_, AssetBalanceOf<T>, ValueQuery, AdscInitTarget<T>>;

    #[pallet::type_value]
    pub fn AdscInitTarget<T: Config>() -> AssetBalanceOf<T> {
        UniqueSaturatedFrom::unique_saturated_from(10_000_000_000 * DPR)
    }

    #[pallet::storage]
    pub type CurrentRewardPeriod<T: Config> = StorageValue<_, u32, ValueQuery, ConstU32<365>>;

    #[pallet::storage]
    pub type BlocklyRewardNum<T: Config> = StorageValue<_, u32, ValueQuery, ConstU32<2>>;

    #[pallet::storage]
    #[pallet::getter(fn saved_day)]
    pub(crate) type SavedDay<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AdscStakerAdd(T::AccountId, u32),
        AdscStakerAddNft(T::AccountId, u32, ClassIdOf<T>, InstanceIdOf<T>),
        RewardPeriod(u32),
        HalfRewardTarget(AssetBalanceOf<T>),
        BaseReward(AssetBalanceOf<T>),
        AdscReward(T::AccountId, AssetBalanceOf<T>),
        BridgeBurned(T::AccountId, H160, AssetBalanceOf<T>),
        BridgeMinted(T::AccountId, H160, AssetBalanceOf<T>),
        BridgeResult { bridge_result: DispatchResult },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not Admin
        NotAdmin,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let _ = T::AdscCurrency::create(
                T::AdscId::get(),
                Self::account_id(),
                true,
                1_000_000_000u32.into(),
            );

            let _ = T::AdscCurrency::set(
                T::AdscId::get(),
                &Self::account_id(),
                b"Adsc".to_vec(),
                b"Adsc".to_vec(),
                18,
            );

            T::DbWeight::get().writes(2u64)
        }

        fn on_initialize(_: T::BlockNumber) -> Weight {
            const MILLISECS_PER_DAY: u64 = 1000 * 3600 * 24;
            const BLOCK_PER_DAY: u32 = 17000;

            let saved_day = Self::saved_day();
            let cur_time: u64 = T::Time::now().unique_saturated_into();

            let mut weight = T::DbWeight::get().reads(2 as u64);
            let cur_day = (cur_time / MILLISECS_PER_DAY) as u32;
            if cur_day > saved_day {
                SavedDay::<T>::put(cur_day);
                let prefix = Self::get_staker_prefix_hash();
                let staker_num = AdscStakers::<T>::count();
                AdscStakerLastKey::<T>::put(prefix);
                BlocklyRewardNum::<T>::put(staker_num / BLOCK_PER_DAY + 2);
                weight += T::DbWeight::get()
                    .reads(2 as u64)
                    .saturating_add(T::DbWeight::get().writes(3 as u64));
            } else {
                weight += Self::adsc_reward(cur_day);
            }

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn add_adsc_staking_account(
            origin: OriginFor<T>,
            account_id: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            let period = CurrentRewardPeriod::<T>::get();

            AdscStakers::<T>::insert(&account_id, period);
            Self::deposit_event(Event::AdscStakerAdd(account_id, period));
            Ok(())
        }

        #[pallet::weight(Weight::from_ref_time(20_000u64))]
        #[transactional]
        pub fn add_adsc_staking_account_with_nft(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            collection_id: ClassIdOf<T>,
            item_id: InstanceIdOf<T>,
            data: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            let period = CurrentRewardPeriod::<T>::get();

            AdscStakers::<T>::insert(&account_id, period);
            AdscNfts::<T>::insert(&account_id, (collection_id, item_id));

            Self::add_nft(collection_id, item_id, account_id.clone(), &data)?;

            Self::deposit_event(Event::AdscStakerAddNft(
                account_id,
                period,
                collection_id,
                item_id,
            ));
            Ok(())
        }

        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn set_reward_period(origin: OriginFor<T>, period: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            CurrentRewardPeriod::<T>::put(period);
            Self::deposit_event(Event::RewardPeriod(period));
            Ok(())
        }

        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn set_half_reward_target(
            origin: OriginFor<T>,
            target: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            CurrentHalfTarget::<T>::put(target);
            Self::deposit_event(Event::HalfRewardTarget(target));
            Ok(())
        }

        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn set_base_reward(
            origin: OriginFor<T>,
            base_reward: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            CurrentAdscBaseReward::<T>::put(base_reward);
            Self::deposit_event(Event::BaseReward(base_reward));
            Ok(())
        }

        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn bridge_burn_adsc(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: H160,
            amount: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let res = Self::do_bridge_burn_adsc(origin, from.clone(), amount);

            Self::deposit_event(Event::BridgeResult { bridge_result: res });
            if res.is_ok() {
                Self::deposit_event(Event::BridgeBurned(from, to, amount));
            }
            Ok(())
        }

        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn bridge_mint_adsc(
            origin: OriginFor<T>,
            from: H160,
            to: T::AccountId,
            amount: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let res = Self::do_bridge_mint_adsc(origin, to.clone(), amount);

            Self::deposit_event(Event::BridgeResult { bridge_result: res });
            if res.is_ok() {
                Self::deposit_event(Event::BridgeMinted(to, from, amount));
            }
            Ok(())
        }
    }

    impl<T: Config> pallet::Pallet<T> {
        pub(crate) fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        fn get_staker_prefix_hash() -> Vec<u8> {
            AdscStakers::<T>::map_storage_final_prefix()
        }

        fn pay_reward(account: &T::AccountId, day: u32) -> DispatchResult {
            let cur_base_val = CurrentAdscBaseReward::<T>::get();
            let portion = Perbill::from_rational(day, CurrentRewardPeriod::<T>::get());
            let real_pay = portion * cur_base_val;
            T::AdscCurrency::mint_into(T::AdscId::get(), account, real_pay)?;
            Self::deposit_event(Event::AdscReward(account.clone(), real_pay));
            let cur_minted = CurrentMintedAdsc::<T>::mutate(|num| {
                *num += real_pay;
                *num
            });
            let cur_hf_target = CurrentHalfTarget::<T>::get();
            if cur_minted >= cur_hf_target {
                CurrentHalfTarget::<T>::put(
                    cur_hf_target.saturating_add(AdscInitTarget::<T>::get()),
                );
                CurrentAdscBaseReward::<T>::mutate(|base| *base = *base / 2u32.into());
            }
            Ok(())
        }

        fn adsc_reward(_cur_day: u32) -> Weight {
            let last_key = Self::adsc_staker_last_key();
            let mut weight = T::DbWeight::get().reads(1 as u64);

            if last_key.is_empty() {
                return weight;
            }
            let mut to_be_removed = Vec::new();
            let mut to_be_sub = Vec::new();
            let mut counter: u32 = 0;
            let mut adsc_iter = AdscStakers::<T>::iter_from(last_key);
            let blockly_num = BlocklyRewardNum::<T>::get();
            let mut last_key = Vec::new();
            weight += T::DbWeight::get().reads(1 as u64);
            loop {
                if let Some((account, period)) = adsc_iter.next() {
                    last_key = AdscStakers::<T>::hashed_key_for(&account);
                    if period == 0 {
                        to_be_removed.push(account);
                    } else {
                        let _ = Self::pay_reward(&account, period);

                        weight += T::DbWeight::get()
                            .reads(3 as u64)
                            .saturating_add(T::DbWeight::get().writes(3 as u64));
                        to_be_sub.push(account);
                    }
                } else {
                    break;
                }
                counter += 1;
                if counter == blockly_num {
                    break;
                }
            }
            AdscStakerLastKey::<T>::put(last_key);
            weight += T::DbWeight::get().writes(1 as u64);
            for account in to_be_removed {
                AdscStakers::<T>::remove(&account);
                if let Some((collection_id, item_id)) = AdscNfts::<T>::take(&account) {
                    let _ = Self::remove_nft(collection_id, item_id);
                }

                weight += T::DbWeight::get().writes(1 as u64);
            }
            for account in to_be_sub {
                weight += T::DbWeight::get().writes(1 as u64);
                AdscStakers::<T>::mutate_exists(&account, |period| {
                    if let Some(ref mut p) = period {
                        *p = p.saturating_sub(1);
                    }
                });
            }
            weight
        }

        pub(crate) fn remove_nft(
            collection_id: ClassIdOf<T>,
            item_id: InstanceIdOf<T>,
        ) -> DispatchResult {
            <pallet_uniques::Pallet<T> as NftMutate<T::AccountId>>::burn(
                &collection_id,
                &item_id,
                None,
            )?;
            pallet_uniques::Pallet::<T>::clear_metadata(
                RawOrigin::Root.into(),
                collection_id,
                item_id,
            )
        }

        pub(crate) fn add_nft(
            collection_id: ClassIdOf<T>,
            item_id: InstanceIdOf<T>,
            account_id: T::AccountId,
            data: &[u8],
        ) -> DispatchResult {
            <pallet_uniques::Pallet<T> as NftMutate<T::AccountId>>::mint_into(
                &collection_id,
                &item_id,
                &account_id,
            )?;
            let data = BoundedVec::truncate_from(data.to_vec());
            pallet_uniques::Pallet::<T>::set_metadata(
                RawOrigin::Root.into(),
                collection_id,
                item_id,
                data,
                false,
            )
        }

        pub(crate) fn do_bridge_burn_adsc(
            origin: OriginFor<T>,
            from: T::AccountId,
            amount: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            T::AdscCurrency::burn_from(T::AdscId::get(), &from, amount)?;
            Ok(())
        }

        pub(crate) fn do_bridge_mint_adsc(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );
            T::AdscCurrency::mint_into(T::AdscId::get(), &to, amount)?;
            Ok(())
        }
    }
}
