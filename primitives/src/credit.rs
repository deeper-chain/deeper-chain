use crate::Vec;
use codec::{Decode, Encode};
use frame_support::weights::Weight;
use scale_info::TypeInfo;
pub use sp_core::H160;
use sp_runtime::{DispatchResult, Percent};

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Each campaign_id represents a DPR Proof-of-Credit promotion campaign.
pub type CampaignId = u16;

/// default reward eras
pub const DEFAULT_REWARD_ERAS: EraIndex = 10 * 365;

pub const OLD_REWARD_ERAS: EraIndex = 270;
// Allow 1 era to increase credit score once
pub const CREDIT_CAP_ONE_ERAS: u64 = 1;

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

impl CreditData {
    pub fn new(campaign_id: CampaignId, credit: u64) -> Self {
        let lv = CreditLevel::get_credit_level(credit);
        CreditData {
            campaign_id,
            credit,
            initial_credit_level: lv,
            current_credit_level: lv,
            reward_eras: DEFAULT_REWARD_ERAS,
            ..Default::default()
        }
    }

    pub fn update(&mut self, credit: u64) {
        let lv = CreditLevel::get_credit_level(credit);
        self.current_credit_level = lv;
        self.credit = credit;
    }

    pub fn update_campaign(&mut self, campaign_id: CampaignId) {
        self.campaign_id = campaign_id;
    }
}

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

impl From<u8> for CreditLevel {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            5 => Self::Five,
            6 => Self::Six,
            7 => Self::Seven,
            8 => Self::Eight,
            _ => Self::Zero,
        }
    }
}

impl Into<u8> for CreditLevel {
    fn into(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
        }
    }
}

impl CreditLevel {
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

    pub fn credit_level_gap(lhs: CreditLevel, rhs: CreditLevel) -> u64 {
        let lhs: u8 = lhs.into();
        let rhs: u8 = rhs.into();
        100u64 * (lhs.saturating_sub(rhs) as u64)
    }
}

pub trait CreditInterface<AccountId, Balance> {
    fn get_credit_score(account_id: &AccountId) -> Option<u64>;
    fn get_evm_credit_score(account_id: &H160) -> Option<u64>;
    fn pass_threshold(account_id: &AccountId) -> bool;
    fn slash_credit(account_id: &AccountId, score: Option<u64>) -> Weight;
    fn get_credit_level(credit_score: u64) -> CreditLevel;
    fn get_reward(
        account_id: &AccountId,
        from: EraIndex,
        to: EraIndex,
    ) -> (Option<Balance>, Weight);
    fn update_credit_by_traffic(server: AccountId);
    fn get_current_era() -> EraIndex;
    fn update_credit_by_tip(who: AccountId, add_credit: u64);
    fn update_credit_by_burn_nft(who: AccountId, add_credit: u64) -> DispatchResult;
    fn init_delegator_history(account_id: &AccountId, era: u32) -> bool;
    fn get_credit_balance(account_id: &AccountId, campaign_id: Option<u16>) -> Vec<Balance>;
    fn add_or_update_credit(account_id: AccountId, credit_score: u64, campaign_id: Option<u16>);
    fn is_first_campaign_end(account_id: &AccountId) -> Option<bool>;
    fn do_unstaking_slash_credit(user: &AccountId) -> DispatchResult;
    fn burn_record(burn_amount: Balance) -> bool;
    fn get_credit_history(account_id: &AccountId) -> Vec<(EraIndex, CreditData)>;
    fn set_staking_balance(account_id: &AccountId, usdt_amount: Balance) -> bool;
    fn get_default_dpr_campaign_id() -> u16;
    fn get_default_usdt_campaign_id() -> u16;
}

impl<AccountId, Balance: From<u32>> CreditInterface<AccountId, Balance> for () {
    fn burn_record(_burn_amount: Balance) -> bool {
        false
    }

    fn get_credit_score(_account_id: &AccountId) -> Option<u64> {
        None
    }
    fn get_evm_credit_score(_account_id: &H160) -> Option<u64> {
        None
    }
    fn pass_threshold(_account_id: &AccountId) -> bool {
        false
    }
    fn slash_credit(_account_id: &AccountId, _score: Option<u64>) -> Weight {
        0
    }
    fn get_credit_level(_credit_score: u64) -> CreditLevel {
        CreditLevel::Zero
    }
    fn get_reward(
        _account_id: &AccountId,
        _from: EraIndex,
        _to: EraIndex,
    ) -> (Option<Balance>, Weight) {
        (None, 0)
    }
    fn update_credit_by_traffic(_server: AccountId) {}
    fn get_current_era() -> EraIndex {
        0
    }
    fn update_credit_by_tip(_who: AccountId, _add_credit: u64) {}
    fn update_credit_by_burn_nft(_who: AccountId, _add_credit: u64) -> DispatchResult {
        Ok(()).into()
    }
    fn init_delegator_history(_account_id: &AccountId, _era: u32) -> bool {
        false
    }
    fn get_credit_balance(_account_id: &AccountId, _campaign_id: Option<u16>) -> Vec<Balance> {
        Vec::new()
    }
    fn add_or_update_credit(_account_id: AccountId, _credit_score: u64, _campaign_id: Option<u16>) {
    }
    fn is_first_campaign_end(_account_id: &AccountId) -> Option<bool> {
        Some(true)
    }
    fn do_unstaking_slash_credit(_user: &AccountId) -> DispatchResult {
        Ok(()).into()
    }
    fn get_credit_history(_account_id: &AccountId) -> Vec<(EraIndex, CreditData)> {
        Vec::new()
    }

    fn set_staking_balance(_account_id: &AccountId, _usdt_amount: Balance) -> bool {
        true
    }

    fn get_default_dpr_campaign_id() -> u16 {
        4u16
    }

    fn get_default_usdt_campaign_id() -> u16 {
        5u16
    }
}
