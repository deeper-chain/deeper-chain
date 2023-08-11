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
pub fn new_genesis_credit_settings<T: crate::Config>() -> Vec<CreditSetting<BalanceOf<T>>> {
    vec![
        CreditSetting {
            campaign_id: 6,
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
            campaign_id: 6,
            credit_level: CreditLevel::One,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(5_000 * DPR),
            base_apy: Percent::from_percent(20) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 1,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Two,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(10_000 * DPR),
            base_apy: Percent::from_percent(30) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 2,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Three,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(20_000 * DPR),
            base_apy: Percent::from_percent(35) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 3,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Four,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(30_000 * DPR),
            base_apy: Percent::from_percent(40) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 7,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Five,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(50_000 * DPR),
            base_apy: Percent::from_percent(45) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 12,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Six,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(60_000 * DPR),
            base_apy: Percent::from_percent(50) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 18,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Seven,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(80_000 * DPR),
            base_apy: Percent::from_percent(55) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 25,
            reward_per_referee: 0u32.into(),
        },
        CreditSetting {
            campaign_id: 6,
            credit_level: CreditLevel::Eight,
            staking_balance: UniqueSaturatedFrom::unique_saturated_from(100_000 * DPR),
            base_apy: Percent::from_percent(60) + Percent::from_percent(10),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 34,
            reward_per_referee: 0u32.into(),
        },
    ]
}

pub fn sub_genesis_apy<T: crate::Config>(subed: u8) -> Vec<CreditSetting<BalanceOf<T>>> {
    let mut tmp = new_genesis_credit_settings::<T>();
    for mut setting in &mut tmp {
        if let Some(apy) = setting.base_apy.checked_sub(&Percent::from_percent(subed)) {
            setting.base_apy = apy;
        }
    }
    tmp
}
