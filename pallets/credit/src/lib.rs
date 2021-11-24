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

use codec::{Decode, Encode};
use sp_runtime::Percent;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

use frame_support::weights::Weight;
use scale_info::TypeInfo;
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq, Copy, Ord, PartialOrd, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CreditLevel {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Default for CreditLevel {
    fn default() -> Self {
        CreditLevel::Zero
    }
}

/// Each campaign_id represents a DPR Proof-of-Credit promotion campaign.
pub type CampaignId = u16;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// settings for a specific campaign_id and credit level
#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CreditSetting<Balance> {
    pub campaign_id: CampaignId,
    pub credit_level: CreditLevel,
    pub staking_balance: Balance,
    pub base_apy: Percent,
    pub bonus_apy: Percent,
    pub max_rank_with_bonus: u32, // max rank which can get bonus in the credit_level
    pub tax_rate: Percent,
    pub max_referees_with_rewards: u8,
    pub reward_per_referee: Balance,
}

#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CreditData {
    pub campaign_id: CampaignId,
    pub credit: u64,
    pub initial_credit_level: CreditLevel,
    pub rank_in_initial_credit_level: u32,
    pub number_of_referees: u8,
    pub current_credit_level: CreditLevel,
    pub reward_eras: EraIndex, // reward eras since device gets online
}

pub trait CreditInterface<AccountId, Balance> {
    fn get_credit_score(account_id: &AccountId) -> Option<u64>;
    fn pass_threshold(account_id: &AccountId) -> bool;
    fn slash_credit(account_id: &AccountId) -> Weight;
    fn get_credit_level(credit_score: u64) -> CreditLevel;
    fn get_reward(
        account_id: &AccountId,
        from: EraIndex,
        to: EraIndex,
    ) -> (Option<(Balance, Balance)>, Weight);
    fn get_top_referee_reward(account_id: &AccountId) -> (Balance, Weight);
    fn update_credit(micropayment: (AccountId, Balance));
    fn update_credit_by_traffic(server: AccountId);
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::Currency;
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
        pallet_prelude::*,
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use pallet_deeper_node::NodeInterface;
    use sp_runtime::{
        traits::{Saturating, Zero},
        Perbill,
    };
    use sp_std::{cmp, collections::btree_map::BTreeMap, convert::TryInto};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Number of blocks per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        /// Currency
        type Currency: Currency<Self::AccountId>;
        /// Credit cap every two eras
        type CreditCapTwoEras: Get<u8>;
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
    }

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_credit)]
    pub type UserCredit<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditData, OptionQuery>;

    /// user credit history is empty until user's device gets onboard   
    #[pallet::storage]
    #[pallet::getter(fn user_credit_history)]
    pub type UserCreditHistory<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(EraIndex, CreditData)>, ValueQuery>;

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
        }
    }

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreditUpdateSuccess(T::AccountId, u64, T::BlockNumber),
        CreditUpdateFailed(T::AccountId, u64, T::BlockNumber),
        CreditSettingUpdated(CreditSetting<BalanceOf<T>>, T::BlockNumber),
        CreditDataAdded(T::AccountId, CreditData, T::BlockNumber),
        CreditDataUpdated(T::AccountId, CreditData, T::BlockNumber),
        CreditScoreIncreased(T::AccountId, u64, T::BlockNumber),
        CreditScoreSlashed(T::AccountId, u64, T::BlockNumber),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// invalid credit data
        InvalidCreditData,
        /// credit data has been initialized
        CreditDataInitialized,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

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
            Self::deposit_event(Event::CreditSettingUpdated(
                credit_setting,
                <frame_system::Pallet<T>>::block_number(),
            ));
            Ok(().into())
        }

        /// update credit data
        #[pallet::weight(<T as pallet::Config>::WeightInfo::add_or_update_credit_data())]
        pub fn add_or_update_credit_data(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            credit_data: CreditData,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::check_credit_data(&credit_data)?;

            let current_block_numbers = <frame_system::Pallet<T>>::block_number();
            if UserCredit::<T>::contains_key(&account_id) {
                UserCredit::<T>::mutate(&account_id, |d| match d {
                    Some(data) => *data = credit_data.clone(),
                    _ => (),
                });
                if !Self::user_credit_history(&account_id).is_empty() {
                    Self::update_credit_history(&account_id, Self::get_current_era());
                }
                Self::deposit_event(Event::CreditDataUpdated(
                    account_id,
                    credit_data,
                    current_block_numbers,
                ));
            } else {
                UserCredit::<T>::insert(&account_id, credit_data.clone());
                Self::deposit_event(Event::CreditDataAdded(
                    account_id,
                    credit_data,
                    current_block_numbers,
                ));
            }
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn slash_offline_device_credit(account_id: &T::AccountId) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            let eras = T::NodeInterface::get_eras_offline(&account_id);
            if eras > 0 && eras % 3 == 0 {
                // slash one credit for being offline every 3 eras
                weight = weight.saturating_add(Self::slash_credit(&account_id));
            }
            weight
        }

        /// inner: update credit score
        fn _update_credit(account_id: &T::AccountId, score: u64) -> bool {
            let current_block_numbers = <frame_system::Pallet<T>>::block_number();
            if UserCredit::<T>::contains_key(account_id) {
                UserCredit::<T>::mutate(account_id, |v| match v {
                    Some(credit_data) => {
                        credit_data.credit = score;
                        credit_data.current_credit_level = Self::get_credit_level(score);
                    }
                    _ => (),
                });
                Self::deposit_event(Event::CreditUpdateSuccess(
                    (*account_id).clone(),
                    score,
                    current_block_numbers,
                ));
                true
            } else {
                Self::deposit_event(Event::CreditUpdateFailed(
                    (*account_id).clone(),
                    score,
                    current_block_numbers,
                ));
                false
            }
        }

        fn update_credit_history(account_id: &T::AccountId, current_era: EraIndex) -> Weight {
            let mut user_credit_history = Self::user_credit_history(&account_id);
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            if !user_credit_history.is_empty() {
                // update credit history only if it's not empty
                let last_index = user_credit_history.len() - 1;
                // user credit data cannot be none unless there is a bug
                let user_credit_data = Self::user_credit(&account_id).unwrap();
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

        fn init_credit_history(account_id: &T::AccountId, credit_data: CreditData) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            match T::NodeInterface::get_onboard_time(account_id) {
                Some(block) => {
                    let onboard_era = Self::block_to_era(block);
                    UserCreditHistory::<T>::insert(account_id, vec![(onboard_era, credit_data)]);
                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
                }
                None => (),
            }
            weight
        }

        fn get_onboard_era(account_id: &T::AccountId) -> Option<EraIndex> {
            match T::NodeInterface::get_onboard_time(account_id) {
                Some(block) => Some(Self::block_to_era(block)),
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

        fn get_current_era() -> EraIndex {
            Self::block_to_era(<frame_system::Pallet<T>>::block_number())
        }

        fn block_to_era(block_number: T::BlockNumber) -> EraIndex {
            TryInto::<u32>::try_into(block_number / T::BlocksPerEra::get())
                .ok()
                .unwrap()
        }

        /// credit data check
        fn check_credit_data(data: &CreditData) -> Result<(), DispatchErrorWithPostInfo> {
            ensure!(
                Self::get_credit_level(data.credit) == data.current_credit_level,
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
    }

    impl<T: Config> CreditInterface<T::AccountId, BalanceOf<T>> for Pallet<T> {
        fn get_credit_score(account_id: &T::AccountId) -> Option<u64> {
            if let Some(credit_data) = Self::user_credit(account_id) {
                Some(credit_data.credit)
            } else {
                None
            }
        }

        /// check if account_id's credit score is pass threshold
        fn pass_threshold(account_id: &T::AccountId) -> bool {
            if let Some(credit_data) = Self::user_credit(account_id) {
                return Self::_pass_threshold(&credit_data);
            }
            false
        }

        fn slash_credit(account_id: &T::AccountId) -> Weight {
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            if UserCredit::<T>::contains_key(account_id) {
                let penalty = T::CreditAttenuationStep::get();
                UserCredit::<T>::mutate(account_id, |v| match v {
                    Some(credit_data) => {
                        credit_data.credit = credit_data.credit.saturating_sub(penalty);
                        credit_data.current_credit_level =
                            Self::get_credit_level(credit_data.credit);

                        Self::deposit_event(Event::CreditScoreSlashed(
                            (*account_id).clone(),
                            (*credit_data).clone().credit,
                            <frame_system::Pallet<T>>::block_number(),
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
            let credit_level = match credit_score {
                0..=99 => CreditLevel::Zero,
                100..=199 => CreditLevel::One,
                200..=299 => CreditLevel::Two,
                300..=399 => CreditLevel::Three,
                400..=499 => CreditLevel::Four,
                500..=599 => CreditLevel::Five,
                600..=699 => CreditLevel::Six,
                700..=799 => CreditLevel::Seven,
                _ => CreditLevel::Eight,
            };
            credit_level
        }

        fn get_reward(
            account_id: &T::AccountId,
            from: EraIndex,
            to: EraIndex,
        ) -> (Option<(BalanceOf<T>, BalanceOf<T>)>, Weight) {
            // silently ignore invalid inputs
            if from > to || to >= Self::get_current_era() {
                return (None, Weight::zero());
            }

            let optional_credit_data = Self::user_credit(account_id); // 1 db read
            let mut weight = T::DbWeight::get().reads_writes(1, 0);
            if optional_credit_data.is_none() {
                return (None, weight);
            }

            let credit_data = optional_credit_data.unwrap();
            if credit_data.reward_eras == 0 {
                return (None, weight);
            }

            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if Self::user_credit_history(account_id).is_empty() {
                weight = weight
                    .saturating_add(Self::init_credit_history(account_id, credit_data.clone()));
            }
            weight = weight.saturating_add(Self::slash_offline_device_credit(account_id));
            let credit_history = Self::user_credit_history(account_id);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if credit_history.is_empty() {
                return (None, weight);
            }

            let onboard_era = credit_history[0].0;
            let expiry_era = onboard_era + credit_data.reward_eras - 1;
            if from > expiry_era {
                return (None, weight);
            }

            let credit_map = Self::get_credit_map(credit_history, from, cmp::min(to, expiry_era));
            if credit_map.is_empty() {
                return (None, weight);
            }

            let mut referee_reward = BalanceOf::<T>::zero();
            let mut poc_reward = BalanceOf::<T>::zero();
            for (credit_data, num_of_eras) in credit_map {
                let initial_credit_level = credit_data.initial_credit_level;
                let credit_setting =
                    Self::credit_settings(credit_data.campaign_id, initial_credit_level.clone());
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
                // referral reward
                let number_of_referees =
                    if credit_data.number_of_referees <= credit_setting.max_referees_with_rewards {
                        credit_data.number_of_referees
                    } else {
                        credit_setting.max_referees_with_rewards
                    };
                let daily_referee_reward = credit_setting
                    .reward_per_referee
                    .saturating_mul(number_of_referees.into());

                // poc reward
                let current_credit_level = credit_data.current_credit_level;
                let (base_daily_poc_reward, daily_poc_reward_with_bonus) =
                    Self::daily_poc_reward(credit_data.campaign_id, current_credit_level.clone());
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

                let daily_poc_reward = if current_credit_level == initial_credit_level {
                    // level unchanged
                    if credit_data.rank_in_initial_credit_level
                        <= credit_setting.max_rank_with_bonus
                    {
                        daily_poc_reward_with_bonus
                    } else {
                        base_daily_poc_reward
                    }
                } else {
                    // level changed
                    let (initial_base_daily_poc_reward, initial_daily_poc_reward_with_bonus) =
                        Self::daily_poc_reward(credit_data.campaign_id, initial_credit_level);
                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
                    if credit_data.rank_in_initial_credit_level
                        <= credit_setting.max_rank_with_bonus
                    {
                        base_daily_poc_reward
                            + (initial_daily_poc_reward_with_bonus - initial_base_daily_poc_reward)
                    } else {
                        base_daily_poc_reward
                    }
                };
                referee_reward = referee_reward
                    .saturating_add(daily_referee_reward.saturating_mul(num_of_eras.into()));
                poc_reward =
                    poc_reward.saturating_add(daily_poc_reward.saturating_mul(num_of_eras.into()));
            }
            (Some((referee_reward, poc_reward)), weight)
        }

        fn get_top_referee_reward(account_id: &T::AccountId) -> (BalanceOf<T>, Weight) {
            let mut weight = T::DbWeight::get().reads_writes(1, 0); // 1 db read for pass_threshold
            if !Self::pass_threshold(account_id) {
                // if not passing threshold
                return (BalanceOf::<T>::zero(), weight);
            }
            let credit_data = Self::user_credit(account_id).unwrap(); // 1 db read
            let credit_setting =
                Self::credit_settings(credit_data.campaign_id, credit_data.initial_credit_level); // 1 db read
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 0));
            let number_of_referees =
                if credit_data.number_of_referees <= credit_setting.max_referees_with_rewards {
                    credit_data.number_of_referees
                } else {
                    credit_setting.max_referees_with_rewards
                };
            let daily_referee_reward = credit_setting
                .reward_per_referee
                .saturating_mul(number_of_referees.into());
            let top_referee_reward =
                daily_referee_reward.saturating_mul(credit_data.reward_eras.into());
            (top_referee_reward, weight)
        }

        /// update credit score based on micropayment tuple
        fn update_credit(micropayment: (T::AccountId, BalanceOf<T>)) {
            let (server_id, balance) = micropayment;
            let balance_num = TryInto::<u128>::try_into(balance).ok().unwrap();
            let mut score_delta: u64 = balance_num
                .checked_div(T::MicropaymentToCreditFactor::get())
                .unwrap_or(0) as u64;
            log!(
                info,
                "server_id: {:?}, balance_num: {}, score_delta:{}",
                server_id.clone(),
                balance_num,
                score_delta
            );
            if score_delta > 0 {
                let onboard_era = Self::get_onboard_era(&server_id);
                if onboard_era.is_none() {
                    // credit is not updated if the device is never online
                    return;
                }
                let current_era = Self::get_current_era();
                // if this is the first update, we use onboard era as the last update era
                let last_credit_update_era =
                    Self::last_credit_update(&server_id).unwrap_or(onboard_era.unwrap());
                let mut eras = (current_era - last_credit_update_era) as u64;
                if eras < 2 && Self::last_credit_update(&server_id).is_none() {
                    // first update within 2 eras, we boost it to 2 eras so that credit can be updated
                    eras = 2;
                }
                if eras >= 2 {
                    let cap: u64 = T::CreditCapTwoEras::get() as u64;
                    let total_cap = cap * (eras / 2);
                    if score_delta > total_cap {
                        score_delta = total_cap;
                        log!(
                            info,
                            "server_id: {:?} score_delta capped at {}",
                            server_id.clone(),
                            total_cap
                        );
                    }

                    let new_credit = Self::get_credit_score(&server_id)
                        .unwrap_or(0)
                        .saturating_add(score_delta);
                    if Self::_update_credit(&server_id, new_credit) {
                        LastCreditUpdate::<T>::insert(&server_id, current_era);
                        Self::update_credit_history(&server_id, current_era);

                        Self::deposit_event(Event::CreditScoreIncreased(
                            server_id,
                            new_credit,
                            <frame_system::Pallet<T>>::block_number(),
                        ));
                    } else {
                        log!(
                            error,
                            "failed to update credit {} for server_id: {:?}",
                            new_credit,
                            server_id.clone()
                        );
                    }
                }
            }
        }

        /// update credit score by traffic
        fn update_credit_by_traffic(server_id: T::AccountId) {
            let onboard_era = Self::get_onboard_era(&server_id);
            if onboard_era.is_none() {
                // credit is not updated if the device is never online
                return;
            }
            let current_era = Self::get_current_era();
            // if this is the first update, we use onboard era as the last update era
            let last_credit_update_era =
                Self::last_credit_update(&server_id).unwrap_or(onboard_era.unwrap());
            let eras = (current_era - last_credit_update_era) as u64;
            if eras >= 2 {
                let cap: u64 = T::CreditCapTwoEras::get() as u64;
                let new_credit = Self::get_credit_score(&server_id)
                    .unwrap_or(0)
                    .saturating_add(cap);
                if Self::_update_credit(&server_id, new_credit) {
                    LastCreditUpdate::<T>::insert(&server_id, current_era);
                    Self::update_credit_history(&server_id, current_era);

                    Self::deposit_event(Event::CreditScoreIncreased(
                        server_id,
                        new_credit,
                        <frame_system::Pallet<T>>::block_number(),
                    ));
                } else {
                    log!(
                        error,
                        "failed to update credit {} for server_id: {:?}",
                        new_credit,
                        server_id.clone()
                    );
                }
            }
        }
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
