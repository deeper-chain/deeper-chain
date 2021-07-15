#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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

#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq)]
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
    pub tax_rate: Percent,
    pub max_referees_with_rewards: u8,
    pub reward_per_referee: Balance,
}

#[derive(Decode, Encode, Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CreditData {
    pub credit: u64,
    pub number_of_referees: u8,
}

pub trait CreditInterface<AccountId> {
    fn get_credit_score(account_id: &AccountId) -> Option<u64>;
    fn pass_threshold(account_id: &AccountId, _ttype: u8) -> bool;
    fn slash_credit(account_id: &AccountId);
    fn get_credit_level(credit_score: u64) -> CreditLevel;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, Vec};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Convert, Saturating};

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_micropayment::Config + pallet_deeper_node::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Number of sessions per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        //type Currency: Currency<Self::AccountId>;
        type CurrencyToVote: Convert<BalanceOf<Self>, u64> + Convert<u128, BalanceOf<Self>>;
        /// Credit init score
        type CreditInitScore: Get<u64>;
        /// Credit score threshold
        type MaxCreditScore: Get<u64>;
        /// Credit score cap per Era
        type CreditScoreCapPerEra: Get<u8>;
        /// credit score attenuation low threshold
        type CreditScoreAttenuationLowerBound: Get<u64>;
        /// credit score attenuation step
        type CreditScoreAttenuationStep: Get<u64>;
        /// Credit score delegated threshold
        type CreditScoreDelegatedPermitThreshold: Get<u64>;
        /// mircropayment size to credit factor:
        type MicropaymentToCreditScoreFactor: Get<u64>;
    }

    pub type BalanceOf<T> = <<T as pallet_micropayment::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn get_user_credit)]
    pub type UserCredit<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditData, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_credit_setting)]
    pub type CreditSettings<T: Config> =
        StorageMap<_, Identity, CreditLevel, CreditSetting<BalanceOf<T>>, ValueQuery>;

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
                <CreditSettings<T>>::insert(cs.credit_level.clone(), cs);
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
        KillCreditSuccess(T::AccountId),
        KillCreditFailed(T::AccountId),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: T::BlockNumber) {
            // We update credit score of account per era
            // Notice: the new_micropayment_size_in_block is block level aggregation, here we need
            // to aggregate total payment and number of clients first before pass it into update_credit's input
            if _n % T::BlocksPerEra::get() == T::BlockNumber::default() {
                // update credit score per era
                let from = _n.saturating_sub(T::BlocksPerEra::get());
                let to = _n.saturating_sub(T::BlockNumber::from(1u32));
                log!(
                    info,
                    "micropayment_statistics block number from {:?} - to {:?}",
                    from,
                    to
                );
                let micropayment_vec =
                    pallet_micropayment::Module::<T>::micropayment_statistics(from, to);
                Self::update_credit(micropayment_vec);

                // attenuate credit score per era
                // Self::attenuate_credit(_n);
            }
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This operation requires sudo now and it will be decentralized in future
        #[pallet::weight(10_000)]
        pub fn update_credit_setting(
            origin: OriginFor<T>,
            credit_setting: CreditSetting<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?; // requires sudo
            CreditSettings::<T>::insert(credit_setting.credit_level.clone(), credit_setting);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// update credit score per era using micropayment vec
        pub fn update_credit(micropayment_vec: Vec<(T::AccountId, BalanceOf<T>, u32)>) {
            for (server_id, balance, size) in micropayment_vec {
                if size >= 1 {
                    let balance_num =
                        <T::CurrencyToVote as Convert<BalanceOf<T>, u64>>::convert(balance);
                    let mut score_delta: u64 = balance_num
                        .checked_div(T::MicropaymentToCreditScoreFactor::get())
                        .unwrap_or(0)
                        .into();
                    log!(
                        info,
                        "server_id: {:?}, balance_num: {}, score_delta:{}",
                        server_id.clone(),
                        balance_num,
                        score_delta
                    );
                    let cap: u64 = T::CreditScoreCapPerEra::get() as u64;
                    if score_delta > cap {
                        score_delta = cap;
                        log!(
                            info,
                            "server_id: {:?} score_delta capped at {}",
                            server_id.clone(),
                            cap
                        );
                    }
                    if score_delta > 0 {
                        let new_credit = Self::get_credit_score(&server_id)
                            .unwrap_or(0)
                            .saturating_add(score_delta);
                        if !Self::_update_credit(&server_id, new_credit) {
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

        /// attenuate credit score per era
        fn attenuate_credit(current_blocknumber: T::BlockNumber) {
            let devices = pallet_deeper_node::Module::<T>::registered_devices();
            for device in devices {
                let server_id = device.account_id;
                let last_update_block =
                    pallet_micropayment::Module::<T>::last_update_block(server_id.clone());
                if current_blocknumber - last_update_block > T::BlocksPerEra::get() {
                    Self::_attenuate_credit(server_id);
                }
            }
        }

        /// inner: attenuate credit score
        fn _attenuate_credit(account_id: T::AccountId) {
            let score = Self::get_credit_score(&account_id).unwrap_or(0);
            let lower_bound = T::CreditScoreAttenuationLowerBound::get();
            if score > lower_bound {
                let attenuated_score = score - T::CreditScoreAttenuationStep::get();
                if attenuated_score > lower_bound {
                    Self::_update_credit(&account_id, attenuated_score);
                } else {
                    Self::_update_credit(&account_id, lower_bound);
                }
            }
        }

        /// clear credit
        fn _kill_credit(account_id: T::AccountId) -> bool {
            if UserCredit::<T>::contains_key(account_id.clone()) {
                UserCredit::<T>::remove(account_id);
                true
            } else {
                false
            }
        }
    }

    impl<T: Config> CreditInterface<T::AccountId> for Module<T> {
        fn get_credit_score(account_id: &T::AccountId) -> Option<u64> {
            if let Some(credit_data) = Self::get_user_credit(account_id) {
                Some(credit_data.credit)
            } else {
                None
            }
        }

        /// check if account_id's credit score is pass threshold ttype
        fn pass_threshold(account_id: &T::AccountId, _ttype: u8) -> bool {
            if UserCredit::<T>::contains_key(account_id) {
                if let Some(score) = Self::get_credit_score(account_id) {
                    if score >= T::CreditScoreDelegatedPermitThreshold::get() {
                        return true;
                    }
                }
            }
            false
        }

        fn slash_credit(account_id: &T::AccountId) {
            if UserCredit::<T>::contains_key(account_id.clone()) {
                let penalty = T::CreditScoreAttenuationStep::get();
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
    }
}
