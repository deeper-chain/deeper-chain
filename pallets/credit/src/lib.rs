#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, ensure};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// mircropayment number to credit factor: /
pub const MICROPAYMENT_TO_CREDIT_SCORE_FACTOR: u64 = 1000;
/// Credit score threshold
pub const CREDIT_SCORE_THRESHOLD: u64 = 100;
/// Credit init score
pub const CREDIT_INIT_SCORE: u64 = 30;
/// credit score attenuation low threshold
pub const CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD: u64 = 40;
/// credit score attenuation step
pub const CREDIT_SCORE_ATTENUATION_STEP: u64 = 5;

// Credit score delegated threshold
pub const CREDIT_SCORE_DELEGATED_PERMIT_THRESHOLD: u64 = 60;
/// per credit score vote weight
pub const TOKEN_PER_CREDIT_SCORE: u64 = 10_000_000;

pub type BlockNumber = u32;

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

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Number of sessions per era.
    type BlocksPerEra: Get<<Self as frame_system::Trait>::BlockNumber>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as Credit {
        //store credit score using map
        pub UserCredit get(fn get_user_credit): map hasher(blake2_128_concat) T::AccountId => Option<u64>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        CreditInitSuccess(AccountId, u64),
        CreditInitFailed(AccountId, u64),
        CreditUpdateSuccess(AccountId, u64),
        CreditUpdateFailed(AccountId, u64),
        KillCreditSuccess(AccountId),
        KillCreditFailed(AccountId),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        AlreadyInitilized,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

                /// Blocks  per era.
                const BlocksPerEra: T::BlockNumber = T::BlocksPerEra::get();


        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        // init credit score
        #[weight = 10_000]
        pub fn initialize_credit_extrinsic(origin, credit:u64) -> dispatch::DispatchResult{ //todo
            let sender = ensure_signed(origin)?;
            let res = Self::initialize_credit(sender.clone(), credit);
            if res == true{
                Self::deposit_event(RawEvent::CreditInitSuccess(sender, credit));
                Ok(())
            }else{
                Self::deposit_event(RawEvent::CreditInitFailed(sender, credit));
                Err(dispatch::DispatchError::Other(
                    "CreditInitFailed",
                ))
            }
            
        }

        // clear credit score
        #[weight = 10_000]
        pub fn kill_credit_extrinsic(origin) -> dispatch::DispatchResult{
            let sender = ensure_signed(origin)?;
            let res = Self::kill_credit(sender.clone());
            if res == true {
                Self::deposit_event(RawEvent::KillCreditSuccess(sender));
                Ok(())
            }else{
                Self::deposit_event(RawEvent::KillCreditFailed(sender));
                Err(dispatch::DispatchError::Other(
                    "KillCreditFailed",
                ))
            }


        }

        // update credit score
        #[weight = 10_000]
        pub fn update_credit_extrinsic(origin, credit:u64) -> dispatch::DispatchResult{ //todo
            let sender = ensure_signed(origin)?;
            let res = Self::update_credit(sender.clone(), credit);
            if res == true {
                Self::deposit_event(RawEvent::CreditUpdateSuccess(sender, credit));
                Ok(())
            }else{
                Self::deposit_event(RawEvent::CreditUpdateFailed(sender, credit));
                Err(dispatch::DispatchError::Other(
                    "CreditUpdateFailed",
                ))
            }
            
        }

        // Anything that needs to be done at the end of the block.
        fn on_finalize(_n: T::BlockNumber) {
            log!(info, "update credit score in block number {:?}", _n);

            // We update credit score per block

            // call attenuate_credit per era
            if _n % T::BlocksPerEra::get() == T::BlockNumber::default(){
                // to call attenuate_credit()

            }
        }
    }
}

impl<T: Trait> Module<T> {
    /// init credit score
    fn initialize_credit(account_id: T::AccountId, score: u64) -> bool {
        if !UserCredit::<T>::contains_key(account_id.clone())
            && score >= CREDIT_INIT_SCORE
            && score <= CREDIT_SCORE_THRESHOLD
        {
            UserCredit::<T>::insert(account_id.clone(), score);
            true
        } else {
            false
        }
    }
    /// update credit score
    fn update_credit(account_id: T::AccountId, score: u64) -> bool {
        if UserCredit::<T>::contains_key(account_id.clone())
            && score >= CREDIT_INIT_SCORE
            && score <= CREDIT_SCORE_THRESHOLD
        {
            UserCredit::<T>::insert(account_id, score);
            true
        } else {
            false
        }
    }

    fn attenuate_credit(account_id: T::AccountId) -> bool {
        let score = Self::get_user_credit(account_id.clone()).unwrap_or(0);
        if score > CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD {
            if score - CREDIT_SCORE_ATTENUATION_STEP >= CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD {
                UserCredit::<T>::insert(account_id, score - CREDIT_SCORE_ATTENUATION_STEP);
            } else {
                UserCredit::<T>::insert(account_id, CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD);
            }
            true
        } else {
            false
        }
    }

    /// clear credit
    fn kill_credit(account_id: T::AccountId) -> bool {
        if UserCredit::<T>::contains_key(account_id.clone()) {
            UserCredit::<T>::remove(account_id);
            true
        } else {
            false
        }
    }
}

pub trait CreditInterface<AccountId> {
    fn get_credit_score(account_id: AccountId) -> Option<u64>;
    fn pass_threshold(account_id: AccountId, _ttype: u8) -> bool;
}

impl<T: Trait> CreditInterface<T::AccountId> for Module<T> {
    fn get_credit_score(account_id: T::AccountId) -> Option<u64> {
        Self::get_user_credit(account_id)
    }

    /// check if account_id's credit score is pass threshold ttype
    fn pass_threshold(account_id: T::AccountId, _ttype: u8) -> bool {
        if UserCredit::<T>::contains_key(account_id.clone()) {
            if let Some(score) = UserCredit::<T>::get(account_id) {
                if score > CREDIT_SCORE_DELEGATED_PERMIT_THRESHOLD {
                    return true;
                }
            }
        }
        false
    }
}
