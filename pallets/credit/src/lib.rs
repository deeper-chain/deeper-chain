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
pub use weights::WeightInfo;

// import section of definition for  data struct and traits
pub mod data_struct_and_traits;
pub use data_struct_and_traits::*;

// import section for calculate the reward
mod reward_manager;
pub use reward_manager::*;

// import section for calculate the reward
mod helper_methods;
pub use helper_methods::*;

mod genesis_for_staking_mock;
pub use genesis_for_staking_mock::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, Vec};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, weights::Weight};
    use frame_system::pallet_prelude::*;
    use pallet_deeper_node::NodeInterface;
    use sp_runtime::traits::{Saturating, Zero};
    use sp_std::{cmp, convert::TryInto};

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
    #[pallet::generate_store(pub (super) trait Store)]
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
    pub type LastCreditUpdateForMicroPayment<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, EraIndex, OptionQuery>;

    /// record the last end era of the campaign in which the Account is currently participating
    #[pallet::storage]
    #[pallet::getter(fn latest_expiry)]
    pub type LatestExpiryInCampaigns<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, EraIndex, OptionQuery>;

    /// record the number of eras remaining for the campaign the Account has participated in.
    #[pallet::storage]
    #[pallet::getter(fn campaign_participated_info)]
    pub type CampaignErasInfo<T: Config> = StorageDoubleMap<
        _,
        Identity,
        T::AccountId,
        Identity,
        CampaignId,
        CampaignErasData,
        OptionQuery,
    >;

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
                <UserCredit<T>>::insert(uc.0.clone(), uc.1.clone());
            }
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreditUpdateSuccess(T::AccountId, u64, T::BlockNumber),
        CreditUpdateFailed(T::AccountId, u64, T::BlockNumber),
        CreditSettingUpdated(CreditSetting<BalanceOf<T>>, T::BlockNumber),
        CreditDataAdded(T::AccountId, CreditData, T::BlockNumber),
        CreditDataUpdated(T::AccountId, CreditData, T::BlockNumber),
        CreditSlashed(T::AccountId, T::BlockNumber),
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
        #[pallet::weight(< T as pallet::Config >::WeightInfo::update_credit_setting())]
        pub fn update_credit_setting(
            origin: OriginFor<T>,
            credit_setting: CreditSetting<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?; // requires sudo
            Self::_update_credit_setting(credit_setting.clone());
            Self::deposit_event(Event::CreditSettingUpdated(credit_setting,
                                                            <frame_system::Module<T>>::block_number()));
            Ok(().into())
        }

        /// update credit data
        #[pallet::weight(< T as pallet::Config >::WeightInfo::add_or_update_credit_data())]
        pub fn add_or_update_credit_data(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            credit_data: CreditData,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::check_credit_data(&credit_data)?;

            if UserCredit::<T>::contains_key(&account_id) {
                UserCredit::<T>::mutate(&account_id, |d| match d {
                    Some(data) => *data = credit_data.clone(),
                    _ => (),
                });
                if !Self::user_credit_history(&account_id).is_empty() {
                    Self::update_credit_history(&account_id, Self::current_era());
                }
                Self::deposit_event(Event::CreditDataUpdated(account_id, credit_data,
                                                             <frame_system::Module<T>>::block_number()
                ));
            } else {
                UserCredit::<T>::insert(&account_id, credit_data.clone());
                Self::deposit_event(Event::CreditDataAdded(account_id, credit_data,
                                                           <frame_system::Module<T>>::block_number()
                ));
            }

            Ok(().into())
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
                        let current_block_numbers = <frame_system::Module<T>>::block_number();
                        Self::deposit_event(Event::CreditSlashed(
                            account_id.clone(),
                            current_block_numbers,
                        ));
                    }
                    _ => (),
                });
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
                weight = weight.saturating_add(Self::update_credit_history(
                    account_id,
                    Self::current_era(),
                ));
            }
            weight
        }

        fn get_reward(
            account_id: &T::AccountId,
            from: EraIndex,
            to: EraIndex,
        ) -> (Option<(BalanceOf<T>, BalanceOf<T>)>, Weight) {
            // silently ignore invalid inputs
            if from > to || to >= Self::current_era() {
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

            let (latest_expiry_eras, ret_weight) = Self::fetch_latest_expiry_eras(account_id);
            weight = weight.saturating_add(ret_weight);
            let expiry_era = latest_expiry_eras - 1;

            if from > expiry_era {
                return (None, weight);
            }

            let user_credit_data: CreditData = credit_history[0].1.clone();
            let campgin_id = user_credit_data.campaign_id;
            let (has_init_campaign_eras, w) = Self::existed_campaign_eras(&account_id, campgin_id);
            weight = weight.saturating_add(w);
            if !has_init_campaign_eras {
                weight = weight.saturating_add(Self::fix_campaign_eras(
                    &account_id,
                    from,
                    credit_history.clone(),
                ));
            }

            let (reward_count, w) =
                Self::calculate_reward(account_id, credit_history, from, cmp::min(to, expiry_era));
            weight = weight.saturating_add(w);

            (reward_count, weight)
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
                let onboard_era = Self::onboard_era(&server_id);
                if onboard_era.is_none() {
                    // credit is not updated if the device is never online
                    return;
                }
                let current_era = Self::current_era();
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
                        LastCreditUpdateForMicroPayment::<T>::insert(&server_id, current_era);
                        Self::update_credit_history(&server_id, current_era);
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
            let onboard_era = Self::onboard_era(&server_id);
            if onboard_era.is_none() {
                // credit is not updated if the device is never online
                return;
            }
            let current_era = Self::current_era();
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
                    LastCreditUpdateForMicroPayment::<T>::insert(&server_id, current_era);
                    Self::update_credit_history(&server_id, current_era);
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
