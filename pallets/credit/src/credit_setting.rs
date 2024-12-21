use node_primitives::{
    credit::{CreditLevel, CreditSetting},
    DPR,
};
use sp_runtime::{
    traits::{CheckedSub, UniqueSaturatedFrom},
    Percent,
};
use sp_std::{vec, vec::Vec};

use crate::BalanceOf;

pub fn sub_genesis_apy<T: crate::Config>(subed: u8) -> Vec<CreditSetting<BalanceOf<T>>> {
    let mut tmp = half_campaign7_settings::<T>(Percent::from_percent(10));
    for setting in &mut tmp {
        if let Some(apy) = setting.base_apy.checked_sub(&Percent::from_percent(subed)) {
            setting.base_apy = apy;
        }
    }
    tmp
}

// Defalt credit setting corresponding to campaign 4
pub fn _half_campaign4_settings<T: crate::Config>() -> Vec<CreditSetting<BalanceOf<T>>> {
    vec![
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Zero,
            staking_balance: 0u32.into(),
            base_apy: Percent::from_percent(0),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 0,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::One,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(5_000 * DPR),
            base_apy: Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 1,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Two,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(10_000 * DPR),
            base_apy: Percent::from_percent(15),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 2,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Three,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(20_000 * DPR),
            base_apy: Percent::from_percent(18),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 3,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Four,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(30_000 * DPR),
            base_apy: Percent::from_percent(20),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 7,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Five,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(50_000 * DPR),
            base_apy: Percent::from_percent(23),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 12,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Six,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(60_000 * DPR),
            base_apy: Percent::from_percent(25),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 18,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Seven,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(80_000 * DPR),
            base_apy: Percent::from_percent(28),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 25,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Eight,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(100_000 * DPR),
            base_apy: Percent::from_percent(30),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 34,
            reward_per_referee: 0u32.into(),
        },
    ]
}

pub fn half_campaign8_settings<T: crate::Config>() -> Vec<CreditSetting<BalanceOf<T>>> {
    vec![
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Zero,
            staking_balance: 0u32.into(),
            base_apy: Percent::from_percent(0),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 0,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::One,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(5_000 * DPR),
            base_apy: Percent::from_percent(5),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 1,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Two,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(10_000 * DPR),
            base_apy: Percent::from_percent(8),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 2,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Three,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(20_000 * DPR),
            base_apy: Percent::from_percent(9),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 3,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Four,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(30_000 * DPR),
            base_apy: Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 7,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Five,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(50_000 * DPR),
            base_apy: Percent::from_percent(12),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 12,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Six,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(60_000 * DPR),
            base_apy: Percent::from_percent(13),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 18,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Seven,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(80_000 * DPR),
            base_apy: Percent::from_percent(14),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 25,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 8,
            credit_level: CreditLevel::Eight,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(100_000 * DPR),
            base_apy: Percent::from_percent(15),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 34,
            reward_per_referee: 0u32.into(),
        },
    ]
}

pub fn _half_campaign6_settings<T: crate::Config>(
    addition_apy: Percent,
) -> Vec<CreditSetting<BalanceOf<T>>> {
    let mut new_campaign7_setting = _half_campaign4_settings::<T>();
    for setting in &mut new_campaign7_setting {
        setting.campaign_id = 7;
        if setting.staking_balance != 0u32.into() {
            setting.base_apy = setting.base_apy + addition_apy;
        }
    }
    new_campaign7_setting
}

// half genesis user's apy
pub fn half_campaign7_settings<T: crate::Config>(
    addition_apy: Percent,
) -> Vec<CreditSetting<BalanceOf<T>>> {
    let mut new_campaign7_setting = half_campaign8_settings::<T>();
    for setting in &mut new_campaign7_setting {
        setting.campaign_id = 7;
        if setting.staking_balance != 0u32.into() {
            setting.base_apy = setting.base_apy + addition_apy;
        }
    }
    new_campaign7_setting
}
