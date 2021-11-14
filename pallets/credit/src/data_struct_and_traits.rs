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

pub(crate) const LOG_TARGET: &'static str = "credit";
// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		frame_support::debug::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

use codec::{Decode, Encode};
use sp_runtime::Percent;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/*
#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
*/
use frame_support::weights::Weight;

#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq, Copy, Ord, PartialOrd)]
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
#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq)]
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

#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
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

#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CampaignErasData {
    pub campaign_id: CampaignId,
    pub start_era: EraIndex,
    pub reward_eras: EraIndex,
    pub ending_era: EraIndex,
    pub remaining_eras: EraIndex,
}

pub trait CreditInterface<AccountId, Balance> {
    fn get_credit_score(account_id: &AccountId) -> Option<u64>;
    fn pass_threshold(account_id: &AccountId) -> bool;
    fn slash_credit(account_id: &AccountId) -> Weight;
    fn get_reward(
        account_id: &AccountId,
        from: EraIndex,
        to: EraIndex,
    ) -> (Option<(Balance, Balance)>, Weight);
    fn get_top_referee_reward(account_id: &AccountId) -> (Balance, Weight);
    fn update_credit(micropayment: (AccountId, Balance)); // 考虑从这里移除
    fn update_credit_by_traffic(server: AccountId);
}
