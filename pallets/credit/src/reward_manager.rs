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

use crate::data_struct_and_traits::CampaignId;
use crate::data_struct_and_traits::CreditData;
use crate::data_struct_and_traits::EraIndex;
use crate::pallet::BalanceOf;
use crate::pallet::Config;
use crate::pallet::Pallet;
use frame_support::traits::Get;
use frame_support::weights::Weight;

use frame_support::traits::Vec;
use sp_runtime::traits::{Saturating, Zero};
use sp_std::collections::btree_map::BTreeMap;

impl<T: Config> Pallet<T> {
    pub fn calculate_reward(
        account_id: &T::AccountId,
        credit_history: Vec<(EraIndex, CreditData)>,
        from: EraIndex,
        to: EraIndex,
    ) -> (Option<(BalanceOf<T>, BalanceOf<T>)>, Weight) {
        let mut weight = T::DbWeight::get().reads_writes(0, 0);
        if credit_history.is_empty() {
            return (None, weight);
        }

        let credit_map = Self::get_credit_map(credit_history, from, to);
        if credit_map.is_empty() {
            return (None, weight);
        }

        let mut referee_reward = BalanceOf::<T>::zero();
        let mut poc_reward = BalanceOf::<T>::zero();
        let mut campaign_map = BTreeMap::<CampaignId, u16>::new();

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

            referee_reward = referee_reward
                .saturating_add(daily_referee_reward.saturating_mul(num_of_eras.into()));
            poc_reward =
                poc_reward.saturating_add(daily_poc_reward.saturating_mul(num_of_eras.into()));

            // update remaining eras info
            if !campaign_map.contains_key(&credit_data.campaign_id) {
                let w = Self::refresh_remaining_eras(account_id, credit_data.campaign_id, to);
                weight = weight.saturating_add(w);
                campaign_map.insert(credit_data.campaign_id.clone(), 1);
            }
        }

        (Some((referee_reward, poc_reward)), weight)
    }

    /// get all the credit data passing the threshold for the eras between "from" and "to"
    pub fn get_credit_map(
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
}
