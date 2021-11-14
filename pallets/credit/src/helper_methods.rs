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

use crate::pallet::BalanceOf;
use crate::pallet::CampaignErasInfo;
use crate::pallet::Config;
use crate::pallet::CreditSettings;
use crate::pallet::DailyPocReward;
use crate::pallet::Error;
use crate::pallet::Event;
use crate::pallet::LatestExpiryInCampaigns;
use crate::pallet::Pallet;
use crate::pallet::UserCredit;
use crate::pallet::UserCreditHistory;

use crate::data_struct_and_traits::CampaignErasData;
use crate::data_struct_and_traits::CampaignId;
use crate::data_struct_and_traits::CreditData;
use crate::data_struct_and_traits::CreditInterface;
use crate::data_struct_and_traits::CreditLevel;
use crate::data_struct_and_traits::CreditSetting;
use crate::data_struct_and_traits::EraIndex;

use frame_support::traits::Get;
use pallet_deeper_node::NodeInterface;

use frame_support::{dispatch::DispatchErrorWithPostInfo, pallet_prelude::*, weights::Weight};
use sp_runtime::{traits::Saturating, Perbill};
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto};

use sp_std::prelude::*;

/// staking campaign's eras related segment
impl<T: Config> Pallet<T> {
    /// set the latest campaign expiration point of the account.
    pub fn refresh_latest_expiry_eras(
        campaign_end_eras: EraIndex,
        account_id: &T::AccountId,
    ) -> Weight {
        let (pre_latest_expiry_era, mut weight) = Self::fetch_latest_expiry_eras(account_id);
        if campaign_end_eras > pre_latest_expiry_era {
            //Replace the latest expiration point.
            LatestExpiryInCampaigns::<T>::insert(account_id, campaign_end_eras); //write 1
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
        }
        weight
    }

    /// get the latest campaign expiration point of the account.
    pub fn fetch_latest_expiry_eras(account_id: &T::AccountId) -> (EraIndex, Weight) {
        let mut latest_expiry_eras = Self::latest_expiry(account_id).unwrap_or(0); //  read 1
        let mut weight = T::DbWeight::get().reads_writes(1, 0);

        // adapt to situations where account have previously participated in some campaigns.
        if 0 == latest_expiry_eras {
            let credit_history = Self::user_credit_history(account_id); //  read 1
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if !credit_history.is_empty() {
                latest_expiry_eras = Self::calculate_expiry_era_from_history(&credit_history);
                // write latest_expiry_eras to storage for fix pre campaign that account has participated.
                LatestExpiryInCampaigns::<T>::insert(account_id, latest_expiry_eras); //write 1
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
            }
        }

        (latest_expiry_eras, weight)
    }

    /// calculate the latest stacking campaign expiration point of the account from the credit history.
    pub fn calculate_expiry_era_from_history(
        credit_history: &Vec<(EraIndex, CreditData)>,
    ) -> EraIndex {
        if credit_history.is_empty() {
            return 0;
        }

        let mut expiry_era = 0;
        let mut campaign_map = BTreeMap::<CampaignId, u16>::new();

        for (eras, creditdata) in credit_history {
            if !campaign_map.contains_key(&creditdata.campaign_id) {
                if (eras + creditdata.reward_eras) > expiry_era {
                    expiry_era = eras + creditdata.reward_eras;
                    campaign_map.insert(creditdata.campaign_id.clone(), 1);
                }
            }
        }

        expiry_era
    }

    pub fn existed_campaign_eras(
        account_id: &T::AccountId,
        campaign_id: CampaignId,
    ) -> (bool, Weight) {
        (
            CampaignErasInfo::<T>::contains_key(account_id, campaign_id),
            T::DbWeight::get().reads_writes(1, 0),
        )
    }

    pub fn fix_campaign_eras(
        account_id: &T::AccountId,
        from_era: EraIndex,
        credit_history: Vec<(EraIndex, CreditData)>,
    ) -> Weight {
        let mut weight = T::DbWeight::get().reads_writes(0, 0);
        let mut i = 0;
        while i < credit_history.len() {
            let credit_data = &credit_history[i].1;
            let campaign_id = credit_data.campaign_id.clone();
            let mut campaign_eras_info = CampaignErasData::default();
            campaign_eras_info.start_era = credit_history[i].0;
            campaign_eras_info.reward_eras = credit_data.reward_eras;
            campaign_eras_info.ending_era =
                campaign_eras_info.start_era + campaign_eras_info.reward_eras;
            campaign_eras_info.remaining_eras = campaign_eras_info.reward_eras;

            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));
            if !CampaignErasInfo::<T>::contains_key(account_id, campaign_id) {
                if campaign_eras_info.start_era < from_era {
                    campaign_eras_info.remaining_eras =
                        campaign_eras_info.ending_era - from_era - 1;
                    if campaign_eras_info.remaining_eras > 0 {
                        CampaignErasInfo::<T>::insert(account_id, campaign_id, campaign_eras_info);
                        weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
                    }
                } else {
                    CampaignErasInfo::<T>::insert(account_id, campaign_id, campaign_eras_info);
                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
                }
            }
            i += 1;
        }

        weight
    }

    pub fn add_campaign_eras(
        account_id: &T::AccountId,
        campaign_id: CampaignId,
        campaign_eras_info: CampaignErasData,
    ) -> Weight {
        let mut weight = T::DbWeight::get().reads_writes(1, 0);
        if !CampaignErasInfo::<T>::contains_key(account_id, campaign_id) {
            CampaignErasInfo::<T>::insert(account_id, campaign_id, campaign_eras_info);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
        }

        weight
    }

    pub fn update_remaining_eras(
        account_id: &T::AccountId,
        campaign_id: CampaignId,
        rewarded_era: EraIndex,
    ) -> Weight {
        let remaining_eras: EraIndex;
        let mut weight = T::DbWeight::get().reads_writes(1, 0);
        match CampaignErasInfo::<T>::get(account_id, campaign_id) {
            Some(eras_info) => {
                remaining_eras = eras_info.reward_eras.saturating_sub(rewarded_era);
                if remaining_eras <= 0 {
                    weight = weight.saturating_add(Self::remove_campaign_participated_info(
                        account_id,
                        campaign_id,
                    ));
                } else {
                    weight = T::DbWeight::get().reads_writes(0, 1);
                    CampaignErasInfo::<T>::mutate(account_id, campaign_id, |v| match v {
                        Some(campaign_eras_info) => {
                            campaign_eras_info.remaining_eras = remaining_eras;
                        }
                        _ => (),
                    });
                }
            }
            _ => (),
        }
        weight
    }

    pub fn remove_campaign_participated_info(
        account_id: &T::AccountId,
        campaign_id: CampaignId,
    ) -> Weight {
        let weight = T::DbWeight::get().reads_writes(0, 1);
        CampaignErasInfo::<T>::remove(account_id, campaign_id);
        weight
    }

    pub fn add_or_update_eras_if_needed(
        account_id: &T::AccountId,
        user_credit_data: CreditData,
        current_era: EraIndex,
    ) -> Weight {
        let mut weight = T::DbWeight::get().reads_writes(1, 0);

        let mut campaign_info = CampaignErasData::default();
        // assume its a new campaign, assign current era to start_era
        campaign_info.start_era = current_era;
        let reward_eras = user_credit_data.reward_eras;
        campaign_info.reward_eras = user_credit_data.reward_eras;
        //saturating_sub
        campaign_info.ending_era = campaign_info
            .start_era
            .saturating_add(campaign_info.reward_eras);
        campaign_info.remaining_eras = campaign_info.reward_eras;
        weight = weight.saturating_add(Self::add_campaign_eras(
            &account_id,
            user_credit_data.campaign_id,
            campaign_info,
        ));

        // update the latest end eras for account.
        weight = weight.saturating_add(Self::refresh_latest_expiry_eras(reward_eras, &account_id));

        weight
    }
}

/// CreditData related methods
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
    pub fn _update_credit(account_id: &T::AccountId, score: u64) -> bool {
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
                <frame_system::Module<T>>::block_number(),
            ));
            true
        } else {
            Self::deposit_event(Event::CreditUpdateFailed(
                (*account_id).clone(),
                score,
                <frame_system::Module<T>>::block_number(),
            ));
            false
        }
    }

    pub fn update_credit_history(account_id: &T::AccountId, current_era: EraIndex) -> Weight {
        let mut user_credit_history = Self::user_credit_history(&account_id);
        let mut weight = T::DbWeight::get().reads_writes(1, 0);

        // update credit history only if it's not empty
        if !user_credit_history.is_empty() {
            // user credit data cannot be none unless there is a bug
            let user_credit_data = Self::user_credit(&account_id).unwrap();
            user_credit_history.push((current_era, user_credit_data.clone()));
            UserCreditHistory::<T>::insert(&account_id, user_credit_history);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

            //If there is a new campaign added, add or update the era-related information.
            weight = weight.saturating_add(Self::add_or_update_eras_if_needed(
                &account_id,
                user_credit_data,
                current_era,
            ));
        }
        weight
    }

    pub fn init_credit_history(account_id: &T::AccountId, credit_data: CreditData) -> Weight {
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
}

/// CreditSetting related methods
impl<T: Config> Pallet<T> {
    pub fn _update_credit_setting(credit_setting: CreditSetting<BalanceOf<T>>) {
        let daily_referee_reward = credit_setting
            .reward_per_referee
            .saturating_mul(credit_setting.max_referees_with_rewards.into());

        // poc reward
        let base_total_reward = Perbill::from_rational_approximation(270u32, 365u32)
            * (credit_setting.base_apy * credit_setting.staking_balance);
        let base_daily_poc_reward = (Perbill::from_rational_approximation(1u32, 270u32)
            * base_total_reward)
            .saturating_sub(daily_referee_reward);

        let base_total_reward_with_bonus = Perbill::from_rational_approximation(270u32, 365u32)
            * (credit_setting
                .base_apy
                .saturating_add(credit_setting.bonus_apy)
                * credit_setting.staking_balance);
        let base_daily_poc_reward_with_bonus = (Perbill::from_rational_approximation(1u32, 270u32)
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

/// helper methods for pallet scope
impl<T: Config> Pallet<T> {
    pub fn onboard_era(account_id: &T::AccountId) -> Option<EraIndex> {
        match T::NodeInterface::get_onboard_time(account_id) {
            Some(block) => Some(Self::block_to_era(block)),
            None => None,
        }
    }

    pub fn _pass_threshold(credit_data: &CreditData) -> bool {
        credit_data.credit >= T::MinCreditToDelegate::get()
    }

    pub fn current_era() -> EraIndex {
        Self::block_to_era(<frame_system::Module<T>>::block_number())
    }

    fn block_to_era(block_number: T::BlockNumber) -> EraIndex {
        TryInto::<u32>::try_into(block_number / T::BlocksPerEra::get())
            .ok()
            .unwrap()
    }

    /// credit data check
    pub fn check_credit_data(data: &CreditData) -> Result<(), DispatchErrorWithPostInfo> {
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

    pub fn get_credit_level(credit_score: u64) -> CreditLevel {
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
}
