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

#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq)]
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

pub trait CreditInterface<AccountId> {
    fn get_credit_score(account_id: AccountId) -> Option<u64>;
    fn pass_threshold(account_id: &AccountId, _ttype: u8) -> bool;
    fn credit_slash(accouont_id: AccountId);
    fn get_credit_level(credit_score: u16) -> CreditLevel;
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
    pub(super) type UserCredit<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreditInitSuccess(T::AccountId, u64),
        CreditInitFailed(T::AccountId, u64),
        CreditUpdateSuccess(T::AccountId, u64),
        CreditUpdateFailed(T::AccountId, u64),
        KillCreditSuccess(T::AccountId),
        KillCreditFailed(T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Value not exists
        NoneValue,
        /// Storage overflow
        StorageOverflow,
        /// account credit already initialized
        AlreadyInitilized,
        /// invalid credit score
        InvalidScore,
        /// credit init failed
        CreditInitFailed,
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
        // init credit score
        #[pallet::weight(10_000)]
        pub fn initialize_credit(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            //todo
            let sender = ensure_signed(origin)?;
            let res = Self::_initialize_credit(&sender, T::CreditInitScore::get());
            if res == true {
                Self::deposit_event(Event::CreditInitSuccess(sender, T::CreditInitScore::get()));
                Ok(().into())
            } else {
                Self::deposit_event(Event::CreditInitFailed(sender, T::CreditInitScore::get()));
                Err(Error::<T>::CreditInitFailed)?
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// init credit score
        pub fn _initialize_credit(account_id: &T::AccountId, score: u64) -> bool {
            // in general, a user start from initial score = 0; with coupon, a user can
            // start from initial score at most CreditInitScore
            // TODO: i.e. add coupon verification for non-zero init credit score
            if !UserCredit::<T>::contains_key(account_id) && score <= T::CreditInitScore::get() {
                UserCredit::<T>::insert(account_id, score);
                true
            } else {
                false
            }
        }

        /// update credit score per era using micropayment vec
        pub fn update_credit(micropayment_vec: Vec<(T::AccountId, BalanceOf<T>, u32)>) {
            for (server_id, balance, size) in micropayment_vec {
                if size >= 1 {
                    let balance_num =
                        <T::CurrencyToVote as Convert<BalanceOf<T>, u64>>::convert(balance);
                    let mut score_delta: u64 = balance_num
                        .checked_div(T::MicropaymentToCreditScoreFactor::get())
                        .unwrap_or(0);
                    let cap: u64 = T::CreditScoreCapPerEra::get() as u64;
                    if score_delta > cap {
                        score_delta = cap
                    }
                    log!(
                        info,
                        "server_id: {:?}, balance_num: {},score_delta:{}",
                        server_id.clone(),
                        balance_num,
                        score_delta
                    );
                    Self::_update_credit(
                        &server_id,
                        Self::get_user_credit(&server_id).unwrap_or(T::CreditInitScore::get())
                            + score_delta,
                    );
                }
            }
        }

        /// innner: update credit score
        fn _update_credit(account_id: &T::AccountId, score: u64) -> bool {
            if UserCredit::<T>::contains_key(account_id) {
                match score {
                    score if score > T::MaxCreditScore::get() => {
                        UserCredit::<T>::insert(account_id, T::MaxCreditScore::get());
                        true
                    }
                    _ => {
                        UserCredit::<T>::insert(account_id, score);
                        true
                    }
                }
            } else {
                // uninitialize case
                Self::_initialize_credit(&account_id, 0);
                Self::_update_credit(account_id, score)
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
        fn _attenuate_credit(account_id: T::AccountId) -> bool {
            let score = Self::get_user_credit(account_id.clone()).unwrap_or(0);
            if score > T::CreditScoreAttenuationLowerBound::get() {
                if score - T::CreditScoreAttenuationStep::get()
                    >= T::CreditScoreAttenuationLowerBound::get()
                {
                    UserCredit::<T>::insert(
                        account_id,
                        score - T::CreditScoreAttenuationStep::get(),
                    );
                } else {
                    UserCredit::<T>::insert(account_id, T::CreditScoreAttenuationLowerBound::get());
                }
                true
            } else {
                false
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
        fn get_credit_score(account_id: T::AccountId) -> Option<u64> {
            Self::get_user_credit(account_id)
        }

        /// check if account_id's credit score is pass threshold ttype
        fn pass_threshold(account_id: &T::AccountId, _ttype: u8) -> bool {
            if UserCredit::<T>::contains_key(account_id) {
                if let Some(score) = UserCredit::<T>::get(account_id) {
                    if score >= T::CreditScoreDelegatedPermitThreshold::get() {
                        return true;
                    }
                }
            }
            false
        }

        /// credit slash
        fn credit_slash(account_id: T::AccountId) {
            if UserCredit::<T>::contains_key(account_id.clone()) {
                UserCredit::<T>::mutate(account_id, |s| {
                    let score = (*s).unwrap_or(0);
                    *s = Some(score.saturating_sub(T::CreditScoreAttenuationStep::get() * 2))
                });
            }
        }

        fn get_credit_level(credit_score: u16) -> CreditLevel {
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
