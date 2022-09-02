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
        Currency, ExistenceRequirement, OnUnbalanced, UnixTime, WithdrawReasons,
    };
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, transactional, weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use node_primitives::credit::{
        CampaignId, CreditData, CreditInterface, CreditLevel, CreditSetting, EraIndex,
        CREDIT_CAP_ONE_ERAS, DEFAULT_REWARD_ERAS, OLD_REWARD_ERAS,
    };
    use node_primitives::{
        deeper_node::NodeInterface,
        user_privileges::{Privilege, UserPrivilegeInterface},
        DPR,
    };
    use scale_info::prelude::string::{String, ToString};
    use sp_core::H160;
    use sp_runtime::{
        traits::{One, Saturating, UniqueSaturatedFrom, Zero},
        Perbill, Percent,
    };
    use sp_std::{cmp, collections::btree_map::BTreeMap, convert::TryInto, prelude::*};

    #[cfg(feature = "std")]
    use frame_support::traits::GenesisBuild;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_uniques::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Number of blocks per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        /// Currency
        type Currency: Currency<Self::AccountId>;
        /// credit attenuation step
        type CreditAttenuationStep: Get<u64>;
        /// Minimum credit to delegate
        type MinCreditToDelegate: Get<u64>;
        /// mircropayment to credit factor:
        type MicropaymentToCreditFactor: Get<u128>;
        /// NodeInterface of deeper-node pallet
        type NodeInterface: NodeInterface<Self::AccountId, Self::BlockNumber>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        type UnixTime: UnixTime;

        type SecsPerBlock: Get<u32>;

        type DPRPerCreditBurned: Get<BalanceOf<Self>>;

        type BurnedTo: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// query user prvileges
        type UserPrivilegeInterface: UserPrivilegeInterface<Self::AccountId>;

        #[pallet::constant]
        type MaxBurnCreditPerAddress: Get<u32>;
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    pub type ClassIdOf<T> = <T as pallet_uniques::Config>::CollectionId;
    pub type InstanceIdOf<T> = <T as pallet_uniques::Config>::ItemId;

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum Releases {
        V1_0_0,
        V2_0_0,
        V3_0_0,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_credit)]
    pub type UserCredit<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditData, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn user_staking_credit)]
    pub type UserStakingCredit<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    /// user credit history is empty until user's device gets onboard   
    #[pallet::storage]
    #[pallet::getter(fn user_credit_history)]
    pub type UserCreditHistory<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(EraIndex, CreditData)>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_daily_burn_dpr)]
    pub type TotalDailyBurnDPR<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_burn_dpr)]
    pub(crate) type TotalBurnDPR<T> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn credit_settings)]
    pub type CreditSettings<T: Config> = StorageDoubleMap<
        _,
        Identity,
        CampaignId,
        Identity,
        CreditLevel,
        CreditSetting<BalanceOf<T>>,
        ValueQuery,
    >;

    /// (daily_base_poc_reward, daily_poc_reward_with_bonus)
    #[pallet::storage]
    #[pallet::getter(fn daily_poc_reward)]
    pub type DailyPocReward<T: Config> = StorageDoubleMap<
        _,
        Identity,
        CampaignId,
        Identity,
        CreditLevel,
        (BalanceOf<T>, BalanceOf<T>),
        ValueQuery,
    >;

    /// record the latest era when user updates the credit with micro-payment    
    #[pallet::storage]
    #[pallet::getter(fn last_credit_update)]
    pub type LastCreditUpdate<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, EraIndex, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_credit_update_timestamp)]
    pub type LastCreditUpdateTimestamp<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn mining_machine_class_credit)]
    pub type MiningMachineClassCredit<T: Config> =
        StorageMap<_, Twox64Concat, ClassIdOf<T>, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn campaign_id_switch)]
    pub type CampaignIdSwitch<T: Config> =
        StorageMap<_, Twox64Concat, CampaignId, CampaignId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn switch_accounts)]
    pub type NotSwitchAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn credit_balances)]
    pub type CreditBalances<T: Config> =
        StorageValue<_, Vec<BalanceOf<T>>, ValueQuery, CreditDefaultBalance<T>>;

    #[pallet::storage]
    #[pallet::getter(fn credit_from_burn_nft)]
    pub type CreditFromBurnNft<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, u64, ValueQuery>;

    #[pallet::type_value]
    pub fn CreditDefaultBalance<T: Config>() -> Vec<BalanceOf<T>> {
        vec![
            UniqueSaturatedFrom::unique_saturated_from(1_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(5_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(10_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(20_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(30_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(50_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(60_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(80_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(100_000 * DPR),
        ]
    }

    #[pallet::storage]
    #[pallet::getter(fn usdt_credit_balances)]
    pub type UsdtCreditBalances<T: Config> =
        StorageValue<_, Vec<BalanceOf<T>>, ValueQuery, UsdtCreditDefaultBalance<T>>;

    #[pallet::type_value]
    pub fn UsdtCreditDefaultBalance<T: Config>() -> Vec<BalanceOf<T>> {
        vec![
            UniqueSaturatedFrom::unique_saturated_from(50 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(75 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(125 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(200 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(300 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(450 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(600 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(800 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(1_000 * DPR),
        ]
    }

    #[pallet::storage]
    #[pallet::getter(fn genesis_credit_balances)]
    pub type GenesisCreditBalances<T: Config> =
        StorageValue<_, Vec<BalanceOf<T>>, ValueQuery, GenesisDefaultBalance<T>>;

    #[pallet::type_value]
    pub fn GenesisDefaultBalance<T: Config>() -> Vec<BalanceOf<T>> {
        vec![
            UniqueSaturatedFrom::unique_saturated_from(1_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(20_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(46_800 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(76_800 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(138_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(218_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(288_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(368_000 * DPR),
            UniqueSaturatedFrom::unique_saturated_from(468_000 * DPR),
        ]
    }

    #[pallet::type_value]
    pub fn NewUserCampaignId() -> u16 {
        4
    }

    #[pallet::storage]
    #[pallet::getter(fn default_campaign_id)]
    pub(crate) type DefaultCampaignId<T> = StorageValue<_, u16, ValueQuery, NewUserCampaignId>;

    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases>;

    #[pallet::storage]
    #[pallet::getter(fn dpr_price)]
    pub(super) type DprPrice<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn price_diff_rate)]
    pub(super) type PriceDiffRate<T: Config> = StorageValue<_, Percent, OptionQuery>;

    /// tupule (BalanceOf<T>,BalanceOf<T>): first usdt amount, second dpr amount when usdt staking
    #[pallet::storage]
    #[pallet::getter(fn user_staking_balance)]
    pub(super) type UserStakingBalance<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (BalanceOf<T>, BalanceOf<T>), OptionQuery>;

    #[pallet::type_value]
    pub fn UsdtDefaultId() -> u16 {
        5
    }

    #[pallet::storage]
    #[pallet::getter(fn default_usdt_campaign_id)]
    pub(crate) type DefaultUsdtCampaignId<T> = StorageValue<_, u16, ValueQuery, UsdtDefaultId>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub credit_settings: Vec<CreditSetting<BalanceOf<T>>>,
        pub user_credit_data: Vec<(T::AccountId, CreditData)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                credit_settings: Default::default(),
                user_credit_data: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for cs in self.credit_settings.clone().into_iter() {
                Pallet::<T>::_update_credit_setting(cs);
            }
            for uc in self.user_credit_data.clone().into_iter() {
                <UserCredit<T>>::insert(uc.0, uc.1);
            }
            StorageVersion::<T>::put(Releases::V3_0_0);
        }
    }

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreditUpdateSuccess(T::AccountId, u64),
        CreditUpdateFailed(T::AccountId, u64),
        CreditSettingUpdated(CreditSetting<BalanceOf<T>>),
        CreditScoreSlashed(T::AccountId, u64),
        CreditDataAddedByTraffic(T::AccountId, u64),
        CreditDataAddedByTip(T::AccountId, u64),
        CreditDataAddedByBurnNft(T::AccountId, u64),
        //Status: 1-Invalid Inputs; 2-InvalidCreditData; 3-NoReward; 4-InvalidCreditHistory; 5-ExpiryEra; 6-CreditMap is empty;
        GetRewardResult(T::AccountId, EraIndex, EraIndex, u8),
        CreditHistoryUpdateSuccess(T::AccountId, EraIndex),
        CreditHistoryUpdateFailed(T::AccountId, EraIndex),
        BurnForAddCredit(T::AccountId, u64),
        UpdateNftCredit(ClassIdOf<T>, u64),
        UpdateSumOfCreditNftBurnHistory(T::AccountId, u64),
        BurnNft(T::AccountId, ClassIdOf<T>, InstanceIdOf<T>, u64),
        StakingCreditScore(T::AccountId, u64),
        SetAdmin(T::AccountId),
        UnstakingResult(T::AccountId, String),
        DPRPrice(BalanceOf<T>, H160),
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
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            if StorageVersion::<T>::get() == Some(Releases::V2_0_0) {
                let new_campaign: Vec<CreditSetting<BalanceOf<T>>> = vec![
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Zero,
                        staking_balance: BalanceOf::<T>::zero(),
                        base_apy: Percent::from_percent(0),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::One,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(3_000 * DPR),
                        base_apy: Percent::from_percent(20),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Two,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(5_000 * DPR),
                        base_apy: Percent::from_percent(30),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Three,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(8_000 * DPR),
                        base_apy: Percent::from_percent(35),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Four,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(12_000 * DPR),
                        base_apy: Percent::from_percent(40),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Five,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(18_000 * DPR),
                        base_apy: Percent::from_percent(45),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Six,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(24_000 * DPR),
                        base_apy: Percent::from_percent(50),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Seven,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(32_000 * DPR),
                        base_apy: Percent::from_percent(55),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                    CreditSetting {
                        campaign_id: 5,
                        credit_level: CreditLevel::Eight,
                        staking_balance: UniqueSaturatedFrom::unique_saturated_from(40_000 * DPR),
                        base_apy: Percent::from_percent(60),
                        bonus_apy: Percent::from_percent(0),
                        max_rank_with_bonus: 0u32,
                        tax_rate: Percent::from_percent(0),
                        max_referees_with_rewards: 0,
                        reward_per_referee: BalanceOf::<T>::zero(),
                    },
                ];

                for setting in new_campaign {
                    Self::_update_credit_setting(setting);
                }

                StorageVersion::<T>::put(Releases::V3_0_0);
                return T::DbWeight::get().reads_writes(1, 10);
            }
            0
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This operation requires sudo now and it will be decentralized in future
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_credit_setting())]
        pub fn update_credit_setting(
            origin: OriginFor<T>,
            credit_setting: CreditSetting<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?; // requires sudo
            Self::_update_credit_setting(credit_setting.clone());
            Self::deposit_event(Event::CreditSettingUpdated(credit_setting));
            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::force_modify_credit_history())]
        pub fn force_modify_credit_history(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            expected_era: EraIndex,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?; // requires sudo
            if UserCreditHistory::<T>::contains_key(&account_id) {
                let is_success = UserCreditHistory::<T>::mutate(&account_id, |history| {
                    if history.len() > 0 {
                        for i in 0..history.len() {
                            if (i + 1 < history.len()
                                && expected_era >= history[i].0
                                && expected_era < history[i + 1].0)
                                || (i + 1 == history.len() && expected_era >= history[i].0)
                            {
                                // the first i records were creted before delegate, should be removed
                                for _j in 0..i {
                                    history.remove(0);
                                }
                                history[0].0 = expected_era;
                                return true;
                            }
                        }
                    }
                    false
                });
                if is_success {
                    Self::deposit_event(Event::CreditHistoryUpdateSuccess(
                        account_id,
                        expected_era,
                    ));
                    return Ok(().into());
                }
                Self::deposit_event(Event::CreditHistoryUpdateFailed(account_id, expected_era));
                return Err(Error::<T>::BadEraOrHistory)?;
            }
            Self::deposit_event(Event::CreditHistoryUpdateFailed(account_id, expected_era));
            Err(Error::<T>::AccountNotFound)?
        }

        /// update credit data
        /// To be deprecated when external_set_credit_data used
        #[pallet::weight(<T as pallet::Config>::WeightInfo::add_or_update_credit_data())]
        pub fn add_or_update_credit_data(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            credit_data: CreditData,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Self::check_credit_data(&credit_data)?;
            Self::do_add_credit_with_event(account_id, credit_data);
            Ok(())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::burn_for_add_credit())]
        pub fn burn_for_add_credit(
            origin: OriginFor<T>,
            credit_score: u64,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let cur_credit = UserCredit::<T>::get(&sender)
                .ok_or(Error::<T>::InvalidCreditData)?
                .credit;
            let max_credit = {
                let history = UserCreditHistory::<T>::get(&sender);
                if history.is_empty() {
                    T::MinCreditToDelegate::get()
                } else {
                    let max_credit = history
                        .into_iter()
                        .max_by(|x, y| (x.1.credit).cmp(&y.1.credit))
                        .unwrap()
                        .1
                        .credit;
                    if max_credit > T::MinCreditToDelegate::get() {
                        max_credit
                    } else {
                        T::MinCreditToDelegate::get()
                    }
                }
            };

            let target_credit = cur_credit.saturating_add(credit_score);
            if target_credit > max_credit {
                Err(Error::<T>::CreditAddTooMuch)?
            }

            let amount = T::DPRPerCreditBurned::get().saturating_mul((credit_score as u32).into());

            let burned = <T as pallet::Config>::Currency::withdraw(
                &sender,
                amount.into(),
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::KeepAlive,
            )?;
            T::BurnedTo::on_unbalanced(burned);
            Self::_update_credit(&sender, target_credit);
            Self::update_credit_history(&sender, Self::get_current_era());
            Self::burn_record(amount);
            Self::deposit_event(Event::<T>::BurnForAddCredit(sender.clone(), credit_score));

            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_nft_class_credit())]
        pub fn update_nft_class_credit(
            origin: OriginFor<T>,
            class_id: ClassIdOf<T>,
            credit: u64,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            MiningMachineClassCredit::<T>::insert(class_id, credit);

            Self::deposit_event(Event::UpdateNftCredit(class_id, credit));
            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_sum_of_credit_nft_burn_history())]
        pub fn update_sum_of_credit_nft_burn_history(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            credit: u64,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            CreditFromBurnNft::<T>::insert(account_id.clone(), credit);

            Self::deposit_event(Event::UpdateSumOfCreditNftBurnHistory(account_id, credit));
            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::burn_nft())]
        #[transactional]
        pub fn burn_nft(
            origin: OriginFor<T>,
            class_id: ClassIdOf<T>,
            instance_id: InstanceIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            ensure!(
                MiningMachineClassCredit::<T>::contains_key(&class_id),
                Error::<T>::MiningMachineClassCreditNoConfig
            );

            let credit_from_burn_nft = CreditFromBurnNft::<T>::get(&sender);
            let credit = MiningMachineClassCredit::<T>::get(&class_id);

            ensure!(
                credit_from_burn_nft + credit <= T::MaxBurnCreditPerAddress::get().into(),
                Error::<T>::OutOfMaxBurnCreditPerAddress
            );

            pallet_uniques::Pallet::<T>::burn(origin, class_id, instance_id, None)?;

            Self::update_credit_by_burn_nft(sender.clone(), credit)?;

            CreditFromBurnNft::<T>::insert(sender.clone(), credit_from_burn_nft + credit);

            Self::deposit_event(Event::BurnNft(sender, class_id, instance_id, credit));

            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_switch_campaign())]
        pub fn set_switch_campaign(
            origin: OriginFor<T>,
            old_ids: Vec<CampaignId>,
            new_ids: Vec<CampaignId>,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            ensure!(
                old_ids.len() == new_ids.len(),
                Error::<T>::CampaignIdNotMatch
            );
            for i in 0..old_ids.len() {
                CampaignIdSwitch::<T>::insert(old_ids[i], new_ids[i]);
            }
            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_not_switch_accounts())]
        pub fn set_not_switch_accounts(
            origin: OriginFor<T>,
            accounts: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            for id in accounts {
                NotSwitchAccounts::<T>::insert(id, true);
            }
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_credit_balances(
            origin: OriginFor<T>,
            credit_balances: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            CreditBalances::<T>::put(credit_balances);
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_usdt_credit_balances(
            origin: OriginFor<T>,
            credit_balances: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            UsdtCreditBalances::<T>::put(credit_balances);
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(0,1))]
        pub fn set_default_campaign_id(
            origin: OriginFor<T>,
            id: u16,
        ) -> DispatchResultWithPostInfo {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            DefaultCampaignId::<T>::put(id);
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(0,1))]
        pub fn set_user_staking_credit(
            origin: OriginFor<T>,
            user_scores: Vec<(T::AccountId, u64)>,
        ) -> DispatchResult {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);

            for (user, score) in user_scores {
                UserStakingCredit::<T>::insert(user, score);
            }
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,1))]
        pub fn unstaking_slash_credit(origin: OriginFor<T>, user: T::AccountId) -> DispatchResult {
            let admin = ensure_signed(origin)?;
            if !Self::is_admin(&admin) {
                Self::deposit_event(Event::UnstakingResult(
                    admin,
                    "not credit admin".to_string(),
                ));
                return Err(Error::<T>::NotAdmin.into());
            }
            Self::do_unstaking_slash_credit(&user)
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::add_or_update_credit_data())]
        pub fn external_set_credit_data(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            credit_data: CreditData,
        ) -> DispatchResult {
            let admin = ensure_signed(origin)?;
            ensure!(Self::is_admin(&admin), Error::<T>::NotAdmin);
            Self::check_credit_data(&credit_data)?;
            Self::do_add_credit_with_other_event(account_id, credit_data);
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn set_price_diff_rate(
            origin: OriginFor<T>,
            price_diff_rate: Percent,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_admin(&who), Error::<T>::NotAdmin);
            PriceDiffRate::<T>::put(price_diff_rate);
            Ok(().into())
        }

        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_dpr_price())]
        pub fn set_dpr_price(
            origin: OriginFor<T>,
            price: BalanceOf<T>,
            worker: H160,
        ) -> DispatchResult {
            ensure!(price != 0u32.into(), Error::<T>::PriceZero);
            let who = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&who, Privilege::OracleWorker),
                Error::<T>::NotOracleWorker
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
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn unset_staking_balance(
            origin: OriginFor<T>,
            account_id: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Self::is_admin(&who), Error::<T>::NotAdmin);
            UserStakingBalance::<T>::remove(account_id);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_admin(user: &T::AccountId) -> bool {
            T::UserPrivilegeInterface::has_privilege(&user, Privilege::CreditAdmin)
        }

        fn is_evm_credit_operation_address(address: &H160) -> bool {
            T::UserPrivilegeInterface::has_evm_privilege(&address, Privilege::EvmCreditOperation)
        }

        pub fn evm_update_credit(
            caller: &H160,
            evm_user: &H160,
            score: u64,
            add_flag: bool,
        ) -> bool {
            if !Self::is_evm_credit_operation_address(&caller) {
                return false;
            }
            let user = T::NodeInterface::get_accounts_evm_deeper(evm_user);
            if user.is_none() {
                return false;
            }
            let user = user.unwrap();

            if add_flag {
                let credit_data = {
                    match UserCredit::<T>::get(&user) {
                        Some(mut credit_data) => {
                            let new_score = credit_data.credit.saturating_add(score);
                            credit_data.update(new_score);
                            credit_data
                        }
                        None => {
                            // do not init credit data, because entering the default campaign need some contition
                            return false;
                        }
                    }
                };
                Self::do_add_credit_with_event(user, credit_data);
            } else {
                Self::slash_credit(&user, Some(score));
            }
            true
        }

        pub fn slash_offline_device_credit(account_id: &T::AccountId) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            let eras = T::NodeInterface::get_eras_offline(&account_id);
            if eras > 0 && eras % 3 == 0 {
                // slash one credit for being offline every 3 eras
                weight = weight.saturating_add(Self::slash_credit(&account_id, None));
            }
            weight
        }

        /// inner: update credit score
        fn _update_credit(account_id: &T::AccountId, score: u64) -> bool {
            if UserCredit::<T>::contains_key(account_id) {
                UserCredit::<T>::mutate(account_id, |v| match v {
                    Some(credit_data) => {
                        credit_data.credit = score;
                        credit_data.current_credit_level = CreditLevel::get_credit_level(score);
                    }
                    _ => (),
                });
                Self::deposit_event(Event::CreditUpdateSuccess((*account_id).clone(), score));
                true
            } else {
                Self::deposit_event(Event::CreditUpdateFailed((*account_id).clone(), score));
                false
            }
        }

        pub fn update_credit_history(account_id: &T::AccountId, current_era: EraIndex) -> Weight {
            let user_credit_data = Self::user_credit(&account_id).unwrap();
            let mut weight = T::DbWeight::get().reads_writes(1, 0);

            let mut user_credit_history = Self::user_credit_history(&account_id);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

            if !user_credit_history.is_empty() {
                // update credit history only if it's not empty
                let last_index = user_credit_history.len() - 1;
                if user_credit_history[last_index].0 == current_era {
                    user_credit_history[last_index] = (current_era, user_credit_data.clone());
                } else {
                    user_credit_history.push((current_era, user_credit_data));
                }
                UserCreditHistory::<T>::insert(&account_id, user_credit_history);
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }
            weight
        }

        fn init_credit_history(
            account_id: &T::AccountId,
            credit_data: CreditData,
            era: u32,
        ) -> Weight {
            UserCreditHistory::<T>::insert(account_id, vec![(era, credit_data)]);
            T::DbWeight::get().reads_writes(0, 1)
        }

        fn get_onboard_era(account_id: &T::AccountId) -> Option<EraIndex> {
            match T::NodeInterface::get_onboard_time(account_id) {
                Some(block_number) => Some(Self::block_to_era(block_number)),
                None => None,
            }
        }

        /// get all the credit data passing the threshold for the eras between "from" and "to"
        fn get_credit_map(
            credit_history: Vec<(EraIndex, CreditData)>,
            from: EraIndex,
            to: EraIndex,
        ) -> BTreeMap<CreditData, u16> {
            let mut credit_map = BTreeMap::<CreditData, u16>::new();
            let mut i = 0;
            for era in from..to + 1 {
                while i < credit_history.len() {
                    if credit_history[i].0 < era {
                        i += 1;
                    } else {
                        break;
                    }
                }
                // either credit_history[i].0 >= era or i == credit_history.len()
                if credit_history[0].0 > era {
                    // if the first historical credit data is after the era paid for,
                    // then the device came onboard after the era paid for.
                    // we simply ignore the era paid for and continue to the next era
                    continue;
                } else {
                    // we get the credit data at the era or the closed one before the era
                    let credit_data = if i < credit_history.len() && credit_history[i].0 == era {
                        credit_history[i].1.clone()
                    } else {
                        credit_history[i - 1].1.clone()
                    };
                    if Self::_pass_threshold(&credit_data) {
                        if credit_map.contains_key(&credit_data) {
                            credit_map.insert(
                                credit_data.clone(),
                                credit_map.get(&credit_data).unwrap() + 1,
                            );
                        } else {
                            credit_map.insert(credit_data, 1);
                        }
                    }
                }
            }
            credit_map
        }

        fn _pass_threshold(credit_data: &CreditData) -> bool {
            credit_data.credit >= T::MinCreditToDelegate::get()
        }

        fn block_to_era(block_number: T::BlockNumber) -> EraIndex {
            TryInto::<EraIndex>::try_into(block_number / T::BlocksPerEra::get())
                .ok()
                .unwrap()
        }

        /// credit data check
        fn check_credit_data(data: &CreditData) -> DispatchResult {
            ensure!(
                CreditLevel::get_credit_level(data.credit) == data.current_credit_level,
                Error::<T>::InvalidCreditData
            );
            let credit_setting = Self::credit_settings(data.campaign_id, data.initial_credit_level);
            ensure!(
                data.number_of_referees <= credit_setting.max_referees_with_rewards,
                Error::<T>::InvalidCreditData
            );
            Ok(())
        }

        fn _update_credit_setting(credit_setting: CreditSetting<BalanceOf<T>>) {
            let daily_referee_reward = credit_setting
                .reward_per_referee
                .saturating_mul(credit_setting.max_referees_with_rewards.into());

            // poc reward
            let base_total_reward = Perbill::from_rational(270u32, 365u32)
                * (credit_setting.base_apy * credit_setting.staking_balance);
            let base_daily_poc_reward = (Perbill::from_rational(1u32, 270u32) * base_total_reward)
                .saturating_sub(daily_referee_reward);

            let base_total_reward_with_bonus = Perbill::from_rational(270u32, 365u32)
                * (credit_setting
                    .base_apy
                    .saturating_add(credit_setting.bonus_apy)
                    * credit_setting.staking_balance);
            let base_daily_poc_reward_with_bonus = (Perbill::from_rational(1u32, 270u32)
                * base_total_reward_with_bonus)
                .saturating_sub(daily_referee_reward);

            DailyPocReward::<T>::insert(
                credit_setting.campaign_id,
                credit_setting.credit_level.clone(),
                (base_daily_poc_reward, base_daily_poc_reward_with_bonus),
            );
            CreditSettings::<T>::insert(
                credit_setting.campaign_id,
                credit_setting.credit_level.clone(),
                credit_setting,
            );
        }

        /// check the interval between two credit update as long enouth
        /// return (u64,bool):
        /// the first means the inteval of eras ;
        /// the second means if use era for check (tobe deprecated)
        fn check_update_credit_interval(
            server_id: &T::AccountId,
            current_era: EraIndex,
            onboard_era: EraIndex,
            now_as_secs: u64,
        ) -> (u64, bool) {
            let diffs;
            let mut era_used = false;
            if let Some(pre_update_timestamp) = Self::last_credit_update_timestamp(server_id) {
                let era_block_count = TryInto::<u64>::try_into(T::BlocksPerEra::get())
                    .ok()
                    .unwrap();
                let secs_per_block = T::SecsPerBlock::get() as u64;
                diffs = now_as_secs.saturating_sub(pre_update_timestamp)
                    / era_block_count.saturating_mul(secs_per_block);
            } else if let Some(last_credit_update_era) = Self::last_credit_update(&server_id) {
                diffs = current_era.saturating_sub(last_credit_update_era) as u64;
                era_used = true;
            } else {
                // if this is the first update, we use onboard era as the last update era
                diffs = current_era.saturating_sub(onboard_era) as u64;
            }
            (diffs, era_used)
        }

        fn do_switch_campaign(
            who: &T::AccountId,
            mut old_data: CreditData,
            expire_era: u32,
        ) -> bool {
            if NotSwitchAccounts::<T>::contains_key(who) {
                return false;
            }
            let new_id = Self::campaign_id_switch(old_data.campaign_id);
            if new_id.is_none() {
                return false;
            }
            let new_id = new_id.unwrap();

            if old_data.campaign_id == new_id {
                old_data.reward_eras += 180;
            } else {
                old_data.campaign_id = new_id;
                old_data.reward_eras = DEFAULT_REWARD_ERAS;
            }

            UserCredit::<T>::insert(who, old_data);
            Self::update_credit_history(who, expire_era);
            true
        }

        fn do_add_credit(account_id: T::AccountId, credit_data: CreditData) {
            if UserCredit::<T>::contains_key(&account_id) {
                UserCredit::<T>::mutate(&account_id, |d| match d {
                    Some(data) => *data = credit_data.clone(),
                    _ => (),
                });
                if !Self::user_credit_history(&account_id).is_empty() {
                    Self::update_credit_history(&account_id, Self::get_current_era());
                }
            } else {
                UserCredit::<T>::insert(&account_id, credit_data.clone());
            }
        }

        fn do_add_credit_with_event(account_id: T::AccountId, credit_data: CreditData) {
            let credit = credit_data.credit;
            Self::do_add_credit(account_id.clone(), credit_data);
            Self::deposit_event(Event::CreditUpdateSuccess(account_id, credit));
        }

        // using diff event for statistics
        fn do_add_credit_with_other_event(account_id: T::AccountId, credit_data: CreditData) {
            let credit = credit_data.credit;
            Self::do_add_credit(account_id.clone(), credit_data);
            Self::deposit_event(Event::StakingCreditScore(account_id, credit));
        }

        fn calc_usdt_daily_poc_reward(
            account_id: &T::AccountId,
            credit_data: &CreditData,
        ) -> (BalanceOf<T>, Weight) {
            let mut weight = Weight::zero();
            let staking_balance = Self::user_staking_balance(account_id);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if staking_balance.is_none() {
                return (0u32.into(), weight);
            }
            let staking_usdt = staking_balance.unwrap().0;
            let price = Self::dpr_price();
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if price.is_none() {
                return (0u32.into(), weight);
            }
            let dpr_amount =
                staking_usdt / price.unwrap() * UniqueSaturatedFrom::unique_saturated_from(DPR);

            let current_credit_level = credit_data.current_credit_level;
            let credit_setting =
                Self::credit_settings(credit_data.campaign_id, current_credit_level);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

            let daily_poc_reward = credit_setting.base_apy * dpr_amount / 365u32.into();
            (daily_poc_reward, weight)
        }

        fn calc_normal_daily_poc_reward(credit_data: &CreditData) -> (BalanceOf<T>, Weight) {
            let mut weight = Weight::zero();
            let initial_credit_level = credit_data.initial_credit_level;
            let credit_setting =
                Self::credit_settings(credit_data.campaign_id, initial_credit_level);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

            // poc reward
            let current_credit_level = credit_data.current_credit_level;
            let (base_daily_poc_reward, daily_poc_reward_with_bonus) =
                Self::daily_poc_reward(credit_data.campaign_id, current_credit_level);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

            let daily_poc_reward = if current_credit_level == initial_credit_level {
                // level unchanged
                if credit_data.rank_in_initial_credit_level <= credit_setting.max_rank_with_bonus {
                    daily_poc_reward_with_bonus
                } else {
                    base_daily_poc_reward
                }
            } else {
                // level changed
                let (initial_base_daily_poc_reward, initial_daily_poc_reward_with_bonus) =
                    Self::daily_poc_reward(credit_data.campaign_id, initial_credit_level);
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
                if credit_data.rank_in_initial_credit_level <= credit_setting.max_rank_with_bonus {
                    base_daily_poc_reward
                        + (initial_daily_poc_reward_with_bonus - initial_base_daily_poc_reward)
                } else {
                    base_daily_poc_reward
                }
            };
            (daily_poc_reward, weight)
        }

        // both campaign id is dpr staking or usdt staking

        fn is_same_campaign_type(lhs: u16, rhs: u16) -> bool {
            let dpr_campaign_ids = vec![0, 1, 2, 3, 4];
            let usdt_campaign_ids = vec![5];

            if usdt_campaign_ids.contains(&lhs) && usdt_campaign_ids.contains(&rhs) {
                return true;
            } else if dpr_campaign_ids.contains(&lhs) && dpr_campaign_ids.contains(&rhs) {
                return true;
            }
            false
        }
    }

    impl<T: Config> CreditInterface<T::AccountId, BalanceOf<T>> for Pallet<T> {
        fn burn_record(burn_amount: BalanceOf<T>) -> bool {
            let cur_era = Self::get_current_era();

            let mut total_burn_dpr = Self::total_burn_dpr();
            let mut total_daily_burn_dpr = Self::total_daily_burn_dpr(cur_era);

            total_daily_burn_dpr = total_daily_burn_dpr.saturating_add(burn_amount);
            total_burn_dpr = total_burn_dpr.saturating_add(burn_amount);
            TotalBurnDPR::<T>::put(total_burn_dpr);
            TotalDailyBurnDPR::<T>::insert(cur_era, total_daily_burn_dpr);

            return true;
        }

        fn get_credit_balance(
            account: &T::AccountId,
            require_id: Option<u16>,
        ) -> Vec<BalanceOf<T>> {
            let user_campaign_id = Self::user_credit(account).map(|data| data.campaign_id);

            let campaign_id = match (user_campaign_id, require_id) {
                (None, None) => u16::MAX,
                (Some(campaign_id), Some(require_id)) => {
                    if Self::is_same_campaign_type(campaign_id, require_id) {
                        campaign_id
                    } else {
                        u16::MAX
                    }
                }
                (Some(campaign_id), None) => campaign_id,
                (None, Some(require_id)) => require_id,
            };

            match campaign_id {
                0 | 1 => Self::genesis_credit_balances(),
                2 | 4 => Self::credit_balances(),
                5 => Self::usdt_credit_balances(),
                _ => Vec::new(),
            }
        }

        fn add_or_update_credit(
            account_id: T::AccountId,
            credit_gap: u64,
            campaign_id: Option<u16>,
        ) {
            let credit_data = {
                match UserCredit::<T>::get(account_id.clone()) {
                    Some(mut credit_data) => {
                        let new_score = credit_data.credit.saturating_add(credit_gap);
                        credit_data.update(new_score);
                        credit_data
                    }
                    None => {
                        let default_id = campaign_id.unwrap_or(Self::default_campaign_id());
                        CreditData::new(default_id, credit_gap)
                    }
                }
            };
            Self::do_add_credit_with_event(account_id.clone(), credit_data);

            let staking_credit = Self::user_staking_credit(&account_id).unwrap_or(0);
            UserStakingCredit::<T>::insert(account_id, staking_credit + credit_gap);
        }

        fn get_current_era() -> EraIndex {
            Self::block_to_era(<frame_system::Pallet<T>>::block_number())
        }

        fn get_credit_score(account_id: &T::AccountId) -> Option<u64> {
            Self::user_credit(account_id).map(|credit_data| credit_data.credit)
        }

        fn get_evm_credit_score(evm_user: &H160) -> Option<u64> {
            T::NodeInterface::get_accounts_evm_deeper(evm_user).and_then(|account_id| {
                Self::user_credit(account_id).map(|credit_data| credit_data.credit)
            })
        }

        /// check if account_id's credit score is pass threshold
        fn pass_threshold(account_id: &T::AccountId) -> bool {
            if let Some(credit_data) = Self::user_credit(account_id) {
                return Self::_pass_threshold(&credit_data);
            }
            false
        }

        fn slash_credit(account_id: &T::AccountId, score: Option<u64>) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            if UserCredit::<T>::contains_key(account_id) {
                let penalty = score.unwrap_or(T::CreditAttenuationStep::get());
                UserCredit::<T>::mutate(account_id, |v| match v {
                    Some(credit_data) => {
                        credit_data.credit = credit_data.credit.saturating_sub(penalty);
                        credit_data.current_credit_level =
                            CreditLevel::get_credit_level(credit_data.credit);

                        Self::deposit_event(Event::CreditScoreSlashed(
                            (*account_id).clone(),
                            (*credit_data).clone().credit,
                        ));
                        Self::deposit_event(Event::CreditUpdateSuccess(
                            (*account_id).clone(),
                            (*credit_data).clone().credit,
                        ));
                    }
                    _ => (),
                });
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
                weight = weight.saturating_add(Self::update_credit_history(
                    account_id,
                    Self::get_current_era(),
                ));
            }
            weight
        }

        fn get_credit_level(credit_score: u64) -> CreditLevel {
            CreditLevel::get_credit_level(credit_score)
        }

        fn get_reward(
            account_id: &T::AccountId,
            from: EraIndex,
            to: EraIndex,
        ) -> (Option<BalanceOf<T>>, Weight) {
            // silently ignore invalid inputs
            let cur_era = Self::get_current_era();
            if from > to || to >= cur_era {
                Self::deposit_event(Event::GetRewardResult(account_id.clone(), from, to, 1));
                return (None, Weight::zero());
            }

            let optional_credit_data = Self::user_credit(account_id); // 1 db read
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            if optional_credit_data.is_none() {
                Self::deposit_event(Event::GetRewardResult(account_id.clone(), from, to, 2));
                return (None, weight);
            }

            let credit_data = optional_credit_data.unwrap();
            if credit_data.reward_eras == 0 {
                Self::deposit_event(Event::GetRewardResult(account_id.clone(), from, to, 3));
                return (None, weight);
            }

            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if Self::user_credit_history(account_id).is_empty() {
                weight = weight.saturating_add(Self::init_credit_history(
                    account_id,
                    credit_data.clone(),
                    cur_era,
                ));
            }
            // TODO: for those not continue delegating's account, also need slash credit
            weight = weight.saturating_add(Self::slash_offline_device_credit(account_id));
            let credit_history = Self::user_credit_history(account_id);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if credit_history.is_empty() {
                Self::deposit_event(Event::GetRewardResult(account_id.clone(), from, to, 4));
                return (None, weight);
            }
            let delegate_era = credit_history[0].0;
            let expiry_era = delegate_era + credit_data.reward_eras - 1;
            if from == expiry_era {
                // switcch campaign forehead
                Self::do_switch_campaign(account_id, credit_data, expiry_era);
            } else if from > expiry_era {
                Self::deposit_event(Event::GetRewardResult(account_id.clone(), from, to, 5));
                return (None, weight);
            }

            let credit_map = Self::get_credit_map(credit_history, from, cmp::min(to, expiry_era));
            if credit_map.is_empty() {
                Self::deposit_event(Event::GetRewardResult(account_id.clone(), from, to, 6));
                return (None, weight);
            }

            let mut poc_reward = BalanceOf::<T>::zero();
            for (credit_data, num_of_eras) in credit_map {
                let (daily_poc_reward, added_weight) = {
                    if credit_data.campaign_id == USDT_CAMPAIGN_ID {
                        Self::calc_usdt_daily_poc_reward(account_id, &credit_data)
                    } else {
                        Self::calc_normal_daily_poc_reward(&credit_data)
                    }
                };
                weight += added_weight;
                poc_reward =
                    poc_reward.saturating_add(daily_poc_reward.saturating_mul(num_of_eras.into()));
            }
            (Some(poc_reward), weight)
        }

        /// update credit score by traffic
        fn update_credit_by_traffic(server_id: T::AccountId) {
            let onboard_era = Self::get_onboard_era(&server_id);
            if onboard_era.is_none() {
                // credit is not updated if the device is never online
                log!(
                    info,
                    "update_credit_by_traffic account : {:?}, never online",
                    server_id
                );
                return;
            }
            let current_era = Self::get_current_era();
            let now_as_secs = T::UnixTime::now().as_secs();
            let (time_eras, era_used) = Self::check_update_credit_interval(
                &server_id,
                current_era,
                onboard_era.unwrap(),
                now_as_secs,
            );
            if time_eras >= CREDIT_CAP_ONE_ERAS {
                let new_credit = Self::get_credit_score(&server_id)
                    .unwrap_or(0)
                    .saturating_add(One::one());
                if Self::_update_credit(&server_id, new_credit) {
                    LastCreditUpdateTimestamp::<T>::insert(&server_id, now_as_secs);
                    Self::update_credit_history(&server_id, current_era);
                    Self::deposit_event(Event::CreditDataAddedByTraffic(
                        server_id.clone(),
                        new_credit,
                    ));
                } else {
                    log!(
                        error,
                        "failed to update credit {} for server_id: {:?}",
                        new_credit,
                        server_id
                    );
                }
                // clear old
                if era_used {
                    LastCreditUpdate::<T>::remove(server_id);
                }
            }
        }

        fn update_credit_by_tip(who: T::AccountId, add_credit: u64) {
            let onboard_era = Self::get_onboard_era(&who);
            if onboard_era.is_none() {
                // credit is not updated if the device is never online
                log!(
                    info,
                    "update_credit_by_tip account : {:?}, never online",
                    who
                );
                return;
            }
            let current_era = Self::get_current_era();
            let new_credit = Self::get_credit_score(&who)
                .unwrap_or(0)
                .saturating_add(add_credit);

            if Self::_update_credit(&who, new_credit) {
                Self::update_credit_history(&who, current_era);
                Self::deposit_event(Event::CreditDataAddedByTip(who.clone(), new_credit));
            } else {
                log!(
                    error,
                    "failed to update credit {} for who: {:?}",
                    new_credit,
                    who
                );
            }
        }

        fn update_credit_by_burn_nft(who: T::AccountId, add_credit: u64) -> DispatchResult {
            let current_era = Self::get_current_era();
            let new_credit = Self::get_credit_score(&who)
                .unwrap_or(0)
                .saturating_add(add_credit);

            if Self::_update_credit(&who, new_credit) {
                Self::update_credit_history(&who, current_era);
                Self::deposit_event(Event::CreditDataAddedByBurnNft(who.clone(), new_credit));
            } else {
                log!(
                    error,
                    "failed to update credit {} for who: {:?}",
                    new_credit,
                    who
                );
                return Err(Error::<T>::AccountNoExistInUserCredit.into());
            }
            Ok(())
        }

        fn init_delegator_history(account_id: &T::AccountId, era: u32) -> bool {
            let credit_data = Self::user_credit(account_id); // 1 db read
            if credit_data.is_none() {
                log!(
                    error,
                    "failed to init_delegator_history for  {:?}",
                    account_id
                );
                return false;
            }
            Self::init_credit_history(account_id, credit_data.unwrap(), era);
            true
        }

        fn is_first_campaign_end(account_id: &T::AccountId) -> Option<bool> {
            let credit = UserCredit::<T>::get(account_id);
            match credit {
                Some(data) => {
                    if data.reward_eras > OLD_REWARD_ERAS {
                        Some(true)
                    } else {
                        Some(false)
                    }
                }
                None => None,
            }
        }

        fn do_unstaking_slash_credit(user: &T::AccountId) -> DispatchResult {
            let user_clone = user.clone();
            let staking_score = Self::user_staking_credit(user);
            if staking_score.is_none() {
                Self::deposit_event(Event::UnstakingResult(
                    user_clone,
                    "staking credit not set".to_string(),
                ));
                return Err(Error::<T>::StakingCreditNotSet.into());
            }

            let whole_score = Self::get_credit_score(user);
            if whole_score.is_none() {
                Self::deposit_event(Event::UnstakingResult(
                    user_clone,
                    "user credit not exist".to_string(),
                ));
                return Err(Error::<T>::AccountNoExistInUserCredit.into());
            }

            let new_score = whole_score.unwrap().saturating_sub(staking_score.unwrap());
            let camp_id = Self::default_campaign_id();
            // when unstaking,change campaign id to defalut campaign id
            let credit_data = CreditData::new(camp_id, new_score);
            UserCredit::<T>::insert(user, credit_data);
            Self::deposit_event(Event::CreditUpdateSuccess(user_clone, new_score));
            UserStakingCredit::<T>::remove(user);
            Self::update_credit_history(&user, Self::get_current_era());

            Ok(())
        }

        fn get_credit_history(account_id: &T::AccountId) -> Vec<(EraIndex, CreditData)> {
            Self::user_credit_history(account_id)
        }

        fn set_staking_balance(account_id: &T::AccountId, usdt_amount: BalanceOf<T>) -> bool {
            match Self::dpr_price() {
                Some(price) => {
                    let dpr_amount = usdt_amount / price;
                    UserStakingBalance::<T>::mutate(account_id, |balance| match balance {
                        Some(balance) => {
                            balance.0 += usdt_amount;
                            balance.1 += dpr_amount;
                        }
                        _ => {
                            *balance = Some((usdt_amount, dpr_amount));
                        }
                    });
                    true
                }
                None => false,
            }
        }

        fn get_default_dpr_campaign_id() -> u16 {
            Self::default_campaign_id()
        }

        fn get_default_usdt_campaign_id() -> u16 {
            Self::default_usdt_campaign_id()
        }
    }

    #[cfg(feature = "std")]
    impl<T: Config> GenesisConfig<T> {
        /// Direct implementation of `GenesisBuild::build_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
            <Self as GenesisBuild<T>>::build_storage(self)
        }

        /// Direct implementation of `GenesisBuild::assimilate_storage`.
        ///
        /// Kept in order not to break dependency.
        pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
            <Self as GenesisBuild<T>>::assimilate_storage(self, storage)
        }
    }
}
