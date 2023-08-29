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
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
pub mod weights;
pub(crate) const LOG_TARGET: &'static str = "credit";
pub(crate) const USDT_CAMPAIGN_ID: u16 = 5;

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

use frame_support::dispatch::DispatchResult;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{
        fungibles, fungibles::metadata::Mutate as MetaMutate, fungibles::Create,
        fungibles::Inspect, fungibles::Mutate, nonfungibles::Inspect as NftInspect, Currency,
        ExistenceRequirement, OnUnbalanced, Time, UnixTime, WithdrawReasons,
    };
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, transactional, weights::Weight,
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use node_primitives::{
        deeper_node::NodeInterface,
        user_privileges::{Privilege, UserPrivilegeInterface},
        DPR,
    };
    use scale_info::prelude::string::{String, ToString};
    use sp_core::{H160, U256};
    use sp_runtime::{
        traits::{
            AccountIdConversion, One, Saturating, UniqueSaturatedFrom, UniqueSaturatedInto, Zero,
        },
        Perbill, Percent,
    };
    use sp_std::{cmp, collections::btree_map::BTreeMap, convert::TryInto, prelude::*};

    #[cfg(feature = "std")]
    use frame_support::traits::GenesisBuild;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Number of blocks per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        /// Currency
        type Adst: MetaMutate<Self::AccountId> + Mutate<Self::AccountId> + Create<Self::AccountId>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
        /// query user prvileges
        type UserPrivilegeInterface: UserPrivilegeInterface<Self::AccountId>;

        type Nft: NftInspect<Self::AccountId>;

        type Time: Time;

        #[pallet::constant]
        type AdstId: Get<AssetIdOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    pub(crate) type AssetIdOf<T> =
        <<T as Config>::Adst as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

    pub(crate) type AssetBalanceOf<T> =
        <<T as Config>::Adst as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    pub(crate) type CollectionIdOf<T> =
        <<T as Config>::Nft as NftInspect<<T as frame_system::Config>::AccountId>>::CollectionId;

    pub(crate) type ItemIdOf<T> =
        <<T as Config>::Nft as NftInspect<<T as frame_system::Config>::AccountId>>::ItemId;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn adst_stakers)]
    pub type AdstStakers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn adst_staker_last_key)]
    pub(crate) type AdstStakerLastKey<T> = StorageValue<_, Vec<u8>, ValueQuery>;

    #[pallet::storage]
    pub type CurrentStakerNum<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub type CurrentAdstReward<T: Config> = StorageValue<_, AssetBalanceOf<T>, ValueQuery>;

    #[pallet::type_value]
    pub fn AdstInitReward<T: Config>() -> AssetBalanceOf<T> {
        UniqueSaturatedFrom::unique_saturated_from(1560 * DPR)
    }

    #[pallet::storage]
    pub type CurrentMintedAdst<T: Config> = StorageValue<_, AssetBalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type CurrentRewardPeriod<T: Config> = StorageValue<_, u32, ValueQuery, ConstU32<180>>;

    #[pallet::storage]
    pub type BlocklyRewardNum<T: Config> = StorageValue<_, u32, ValueQuery, ConstU32<2>>;

    #[pallet::storage]
    #[pallet::getter(fn saved_day)]
    pub(crate) type SavedDay<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreditUpdateSuccess(T::AccountId, u64),
        CreditUpdateFailed(T::AccountId, u64),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// invalid credit data
        InvalidCreditData,
        /// credit data has been initialized
        CreditDataInitialized,
        /// over history credit max value
        CreditAddTooMuch,
        /// credit history or input era is wrong
        BadEraOrHistory,
        /// account not found
        AccountNotFound,
        /// account not exist in user credit
        AccountNoExistInUserCredit,
        /// mining machine class credit no config
        MiningMachineClassCreditNoConfig,
        /// Campain id switch not match
        CampaignIdNotMatch,
        /// Not Admin
        NotAdmin,
        /// Not OracleWorker
        NotOracleWorker,
        /// Staking credit score not set
        StakingCreditNotSet,
        /// Out of max burn credit per address
        OutOfMaxBurnCreditPerAddress,
        /// price diffs too much
        PriceDiffTooMuch,
        /// price is zero
        PriceZero,
        /// not device admin
        NotDeviceAdmin,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let _ = T::Adst::create(
                T::AdstId::get(),
                Self::account_id(),
                true,
                1_000_000u32.into(),
            );

            Weight::from_ref_time(0)
        }

        fn on_finalize(_: T::BlockNumber) {
            const MILLISECS_PER_DAY: u64 = 1000 * 3600 * 24;
            const BLOCK_PER_DAY: u32 = 17200;

            let saved_day = Self::saved_day();
            let cur_time: u64 = T::Time::now().unique_saturated_into();
            let cur_day = (cur_time / MILLISECS_PER_DAY) as u32;
            if cur_day > saved_day {
                SavedDay::<T>::put(cur_day);
                let prefix = Self::get_account_release_prefix_hash();
                AdstStakerLastKey::<T>::put(prefix);
                let staker_num = CurrentStakerNum::<T>::get();
                BlocklyRewardNum::<T>::put(staker_num / BLOCK_PER_DAY + 2);
            } else {
                Self::adst_reward(cur_day);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(Weight::from_ref_time(10_000u64))]
        pub fn add_adst_staking_account(
            origin: OriginFor<T>,
            account_id: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::CreditAdmin),
                Error::<T>::NotAdmin
            );

            let period = CurrentRewardPeriod::<T>::get();

            AdstStakers::<T>::insert(account_id, period);

            Ok(())
        }
    }

    impl<T: Config> pallet::Pallet<T> {
        pub(crate) fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        fn get_account_release_prefix_hash() -> Vec<u8> {
            use frame_support::storage::generator::StorageMap;
            AdstStakers::<T>::prefix_hash()
        }

        fn pay_reward(account : &T::AccountId, day : u32) {

        }

        fn adst_reward(_cur_day: u32) {
            let last_key = Self::adst_staker_last_key();
            if last_key.is_empty() {
                return;
            }
            let mut to_be_removed = Vec::new();
            let mut to_be_sub = Vec::new();
            let mut counter: u32 = 0;
            let mut adst_iter = AdstStakers::<T>::iter_from(last_key);
            let blockly_num = BlocklyRewardNum::<T>::get();
            let mut last_key = Vec::new();
            loop {
                if let Some((account, period)) = adst_iter.next() {
                    last_key = AdstStakers::<T>::hashed_key_for(&account);
                    if period == 0 {
                        to_be_removed.push(account);
                    } else {
                        Self::pay_reward(&account,period);
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

            AdstStakerLastKey::<T>::put(last_key);
            for account in to_be_removed {
                AdstStakers::<T>::remove(&account);
            }
            for account in to_be_sub {
                AdstStakers::<T>::mutate_exists(&account, |period| {
                    if let Some(ref mut p) = period {
                        *p = p.saturating_sub(1);
                    }
                });
            }
        }
    }
}
