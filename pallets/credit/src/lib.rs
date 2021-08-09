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

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;

use sp_std::prelude::*;
pub use weights::WeightInfo;

#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq, Copy)]
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

#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CreditSetting<Balance> {
    pub credit_level: CreditLevel,
    pub balance: Balance,
    pub base_apy: Percent,
    pub bonus_apy: Percent,
    pub max_rank_with_bonus: u32, // max rank which can get bonus in the credit_level
    pub tax_rate: Percent,
    pub max_referees_with_rewards: u8,
    pub reward_per_referee: Balance,
}

#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CreditData<BlockNumber> {
    pub credit: u64,
    pub initial_credit_level: CreditLevel,
    pub rank_in_initial_credit_level: u32,
    pub number_of_referees: u8,
    pub expiration: BlockNumber,
}

pub trait CreditInterface<AccountId, Balance> {
    fn get_credit_score(account_id: &AccountId) -> Option<u64>;
    fn get_number_of_referees(account_id: &AccountId) -> Option<u8>;
    fn pass_threshold(account_id: &AccountId, _type: u8) -> bool;
    fn slash_credit(account_id: &AccountId);
    fn get_credit_level(credit_score: u64) -> CreditLevel;
    fn get_reward(account_id: &AccountId) -> Option<(Balance, Balance)>;
    fn get_top_referee_reward(account_id: &AccountId) -> Option<Balance>;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, Vec};
    use frame_support::{
        dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
        pallet_prelude::*,
    };
    use frame_system::pallet_prelude::*;
    use pallet_deeper_node::NodeInterface;
    use sp_runtime::{
        traits::{Saturating, Zero},
        Perbill,
    };
    use sp_std::{cmp, convert::TryInto};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_micropayment::Config + pallet_deeper_node::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Number of sessions per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        /// Credit init score
        type InitialCredit: Get<u64>;
        /// Credit cap per Era
        type CreditCapTwoEras: Get<u8>;
        /// credit attenuation step
        type CreditAttenuationStep: Get<u64>;
        /// Minimum credit to delegate
        type MinCreditToDelegate: Get<u64>;
        /// mircropayment to credit factor:
        type MicropaymentToCreditFactor: Get<u128>;
        /// NodeInterface of deeper-node pallet
        type NodeInterface: NodeInterface<Self::AccountId>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    pub type BalanceOf<T> = <<T as pallet_micropayment::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn user_credit)]
    pub type UserCredit<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditData<T::BlockNumber>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn credit_settings)]
    pub type CreditSettings<T: Config> =
        StorageMap<_, Identity, CreditLevel, CreditSetting<BalanceOf<T>>, ValueQuery>;

    /// (daily_base_poc_reward, daily_poc_reward_with_bonus)
    #[pallet::storage]
    #[pallet::getter(fn daily_poc_reward)]
    pub type DailyPocReward<T: Config> =
        StorageMap<_, Identity, CreditLevel, (BalanceOf<T>, BalanceOf<T>), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn last_credit_update)]
    pub type LastCreditUpdate<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub credit_settings: Vec<CreditSetting<BalanceOf<T>>>,
        pub user_credit_data: Vec<(T::AccountId, CreditData<T::BlockNumber>)>,
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
    #[pallet::metadata(T::AccountId = "AccountId")]
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
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: T::BlockNumber) {
            // We update credit score of account per era
            // Notice: the new_micropayment_size_in_block is block level aggregation, here we need
            // to aggregate total payment and number of clients first before pass it into update_credit's input
            if n % T::BlocksPerEra::get() == T::BlockNumber::default() {
                // update credit score per era
                let from = n.saturating_sub(T::BlocksPerEra::get());
                let to = n.saturating_sub(T::BlockNumber::from(1u32));
                log!(
                    info,
                    "micropayment_statistics block number from {:?} - to {:?}",
                    from,
                    to
                );
                let micropayment_vec = pallet_micropayment::Module::<T>::micropayment_statistics();
                Self::update_credit(micropayment_vec);
                Self::slash_offline_devices_credit();
            }
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
            Self::_update_credit_setting(credit_setting);
            Ok(().into())
        }

        /// update creditdata
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_credit_data())]
        pub fn update_credit_data(
            origin: OriginFor<T>,
            account_id: T::AccountId,
            mut credit_data: CreditData<T::BlockNumber>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Self::check_credit_data(&credit_data)?;
            credit_data.expiration = T::BlockNumber::default();

            if UserCredit::<T>::contains_key(&account_id) {
                UserCredit::<T>::mutate(&account_id, |d| match d {
                    Some(data) => *data = credit_data,
                    _ => (),
                });
            } else {
                UserCredit::<T>::insert(&account_id, credit_data);
            }
            Ok(().into())
        }

        /// initialize credit score
        #[pallet::weight(<T as pallet::Config>::WeightInfo::initialize_credit())]
        pub fn initialize_credit(
            origin: OriginFor<T>,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                !UserCredit::<T>::contains_key(&account_id),
                Error::<T>::CreditDataInitialized
            );
            let credit_data = CreditData {
                credit: T::InitialCredit::get(),
                ..Default::default()
            };
            UserCredit::<T>::insert(&account_id, credit_data);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// update credit score per era using micropayment vec
        pub fn update_credit(micropayment_vec: Vec<(T::AccountId, BalanceOf<T>, u32)>) {
            for (server_id, balance, _num_of_clients) in micropayment_vec {
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
                    let current_block = <frame_system::Module<T>>::block_number();
                    let last_credit_update = Self::last_credit_update(&server_id).unwrap_or(
                        <pallet_deeper_node::Module<T>>::onboard_time(&server_id)
                            .unwrap_or(current_block),
                    );
                    let eras = TryInto::<u64>::try_into(
                        (current_block - last_credit_update) / T::BlocksPerEra::get(),
                    )
                    .ok()
                    .unwrap();
                    let cap: u64 = T::CreditCapTwoEras::get() as u64;
                    let total_cap = cmp::max(1, cap * eras / 2); // at least 1
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
                        LastCreditUpdate::<T>::insert(&server_id, current_block);
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

        /// inner: update credit score
        fn _update_credit(account_id: &T::AccountId, score: u64) -> bool {
            if UserCredit::<T>::contains_key(account_id) {
                UserCredit::<T>::mutate(account_id, |v| match v {
                    Some(credit_data) => credit_data.credit = score,
                    _ => (),
                });
                Self::deposit_event(Event::CreditUpdateSuccess((*account_id).clone(), score));
                true
            } else {
                Self::deposit_event(Event::CreditUpdateFailed((*account_id).clone(), score));
                false
            }
        }

        /// credit data check
        fn check_credit_data(
            data: &CreditData<T::BlockNumber>,
        ) -> Result<(), DispatchErrorWithPostInfo> {
            ensure!(
                Self::get_credit_level(data.credit) == data.initial_credit_level,
                Error::<T>::InvalidCreditData
            );
            let credit_setting = Self::credit_settings(data.initial_credit_level);
            ensure!(
                data.number_of_referees <= credit_setting.max_referees_with_rewards,
                Error::<T>::InvalidCreditData
            );
            Ok(())
        }

        pub fn slash_offline_devices_credit() {
            for device in <pallet_deeper_node::Module<T>>::devices_onboard() {
                let days = T::NodeInterface::get_days_offline(&device);
                if days > 0 && days % 3 == 0 {
                    // slash one credit for being offline every 3 days
                    Self::slash_credit(&device);
                }
            }
        }
    }

    impl<T: Config> Module<T> {
        fn _update_credit_setting(credit_setting: CreditSetting<BalanceOf<T>>) {
            let daily_referee_reward = credit_setting
                .reward_per_referee
                .saturating_mul(credit_setting.max_referees_with_rewards.into());

            // poc reward
            let base_total_reward = Perbill::from_rational_approximation(270u32, 365u32)
                * (credit_setting.base_apy * credit_setting.balance);
            let base_daily_poc_reward = (Perbill::from_rational_approximation(1u32, 270u32)
                * base_total_reward)
                .saturating_sub(daily_referee_reward);

            let base_total_reward_with_bonus = Perbill::from_rational_approximation(270u32, 365u32)
                * (credit_setting
                    .base_apy
                    .saturating_add(credit_setting.bonus_apy)
                    * credit_setting.balance);
            let base_daily_poc_reward_with_bonus =
                (Perbill::from_rational_approximation(1u32, 270u32) * base_total_reward_with_bonus)
                    .saturating_sub(daily_referee_reward);

            DailyPocReward::<T>::insert(
                credit_setting.credit_level.clone(),
                (base_daily_poc_reward, base_daily_poc_reward_with_bonus),
            );
            CreditSettings::<T>::insert(credit_setting.credit_level.clone(), credit_setting);
        }
    }

    impl<T: Config> CreditInterface<T::AccountId, BalanceOf<T>> for Module<T> {
        fn get_credit_score(account_id: &T::AccountId) -> Option<u64> {
            if let Some(credit_data) = Self::user_credit(account_id) {
                Some(credit_data.credit)
            } else {
                None
            }
        }

        fn get_number_of_referees(account_id: &T::AccountId) -> Option<u8> {
            if let Some(credit_data) = Self::user_credit(account_id) {
                Some(credit_data.number_of_referees)
            } else {
                None
            }
        }

        /// check if account_id's credit score is pass threshold type
        fn pass_threshold(account_id: &T::AccountId, _type: u8) -> bool {
            if let Some(score) = Self::get_credit_score(account_id) {
                if score >= T::MinCreditToDelegate::get() {
                    return true;
                }
            }
            false
        }

        fn slash_credit(account_id: &T::AccountId) {
            if UserCredit::<T>::contains_key(account_id.clone()) {
                let penalty = T::CreditAttenuationStep::get();
                UserCredit::<T>::mutate(account_id, |v| match v {
                    Some(credit_data) => {
                        credit_data.credit = credit_data.credit.saturating_sub(penalty)
                    }
                    _ => (),
                });
            }
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

        fn get_reward(account_id: &T::AccountId) -> Option<(BalanceOf<T>, BalanceOf<T>)> {
            // read storage
            if Self::pass_threshold(account_id, 0) {
                let credit_data = Self::user_credit(account_id).unwrap();
                let current_block = <frame_system::Module<T>>::block_number();
                let onboard_time =
                    <pallet_deeper_node::Module<T>>::onboard_time(account_id).unwrap();
                if current_block <= onboard_time + credit_data.expiration {
                    // not expired
                    let initial_credit_level = credit_data.initial_credit_level;
                    let credit_setting = Self::credit_settings(initial_credit_level.clone());
                    // referral reward
                    let number_of_referees = if credit_data.number_of_referees
                        <= credit_setting.max_referees_with_rewards
                    {
                        credit_data.number_of_referees
                    } else {
                        credit_setting.max_referees_with_rewards
                    };
                    let daily_referee_reward = credit_setting
                        .reward_per_referee
                        .saturating_mul(number_of_referees.into());

                    // poc reward
                    let current_credit_level = Self::get_credit_level(credit_data.credit); // get current credit_level
                    let (base_daily_poc_reward, daily_poc_reward_with_bonus) =
                        Self::daily_poc_reward(current_credit_level.clone());

                    if current_credit_level == initial_credit_level {
                        // level unchanged
                        let daily_poc_reward = if credit_data.rank_in_initial_credit_level
                            <= credit_setting.max_rank_with_bonus
                        {
                            daily_poc_reward_with_bonus
                        } else {
                            base_daily_poc_reward
                        };
                        Some((daily_referee_reward, daily_poc_reward))
                    } else {
                        // level changed
                        let (initial_base_daily_poc_reward, initial_daily_poc_reward_with_bonus) =
                            Self::daily_poc_reward(initial_credit_level);

                        let daily_poc_reward = if credit_data.rank_in_initial_credit_level
                            <= credit_setting.max_rank_with_bonus
                        {
                            base_daily_poc_reward
                                + (initial_daily_poc_reward_with_bonus
                                    - initial_base_daily_poc_reward)
                        } else {
                            base_daily_poc_reward
                        };
                        Some((daily_referee_reward, daily_poc_reward))
                    }
                } else {
                    // expired
                    // only daily_base_poc_reward
                    let credit_level = Self::get_credit_level(credit_data.credit);
                    let (base_daily_poc_reward, _) = Self::daily_poc_reward(credit_level);
                    Some((BalanceOf::<T>::zero(), base_daily_poc_reward))
                }
            } else {
                None
            }
        }

        fn get_top_referee_reward(account_id: &T::AccountId) -> Option<BalanceOf<T>> {
            if Self::pass_threshold(account_id, 0) {
                let credit_data = Self::user_credit(account_id).unwrap();
                let credit_setting = Self::credit_settings(credit_data.initial_credit_level);
                let number_of_referees =
                    if credit_data.number_of_referees <= credit_setting.max_referees_with_rewards {
                        credit_data.number_of_referees
                    } else {
                        credit_setting.max_referees_with_rewards
                    };
                let daily_referee_reward = credit_setting
                    .reward_per_referee
                    .saturating_mul(number_of_referees.into());
                let top_referee_reward = daily_referee_reward.saturating_mul((270u32).into());
                return Some(top_referee_reward);
            }
            None
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
