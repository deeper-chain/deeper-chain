#![cfg_attr(not(feature = "std"), no_std)]

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
pub const CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD: u64 = 50;

// Credit score delegated threshold
pub const CREDIT_SCORE_DELEGATED_PERMIT_THRESHOLD: u64 = 60;
/// per credit score vote weight
pub const TOKEN_PER_CREDIT_SCORE: u64 = 10_000_000;

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
        KillCreditSuccess(AccountId),
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

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        // An example dispatchable that takes a singles value as a parameter, writes the value to
        // storage and emits an event. This function must be dispatched by a signed extrinsic.

        // Check that the extrinsic was signed and get the signer.
        // This function will return an error if the extrinsic is not signed.
        // https://substrate.dev/docs/en/knowledgebase/runtime/origin

        // init credit score
        #[weight = 10_000]
        pub fn initilize_credit(origin) -> dispatch::DispatchResult{
            let sender = ensure_signed(origin)?;
            ensure!(!UserCredit::<T>::contains_key(sender.clone()), "Credit Score of AccountId  already Initilized");
            UserCredit::<T>::insert(sender.clone(), CREDIT_INIT_SCORE);
            Self::deposit_event(RawEvent::CreditInitSuccess(sender, CREDIT_INIT_SCORE));

            Ok(())
        }

        // clear credit score
        #[weight = 10_000]
        pub fn kill_credit(origin) -> dispatch::DispatchResult{
            let sender = ensure_signed(origin)?;
            ensure!(UserCredit::<T>::contains_key(sender.clone()), "AccountId is not existed");
            UserCredit::<T>::remove(sender.clone());
            Self::deposit_event(RawEvent::KillCreditSuccess(sender));
            Ok(())
        }

        // Anything that needs to be done at the end of the block.
		fn on_finalize(_n: T::BlockNumber) {
			// We update credit score here.
			log!(info, "update credit score in block number {:?}", _n);
		}
    }
}

pub trait CreditInterface<AccountId> {
    fn initilize_credit(account_id: AccountId, score: u64) -> bool;
    fn update_credit(account_id: AccountId, score: u64) -> bool;
    fn pass_threshold(account_id: AccountId, ttype: u8) -> bool;
    fn credit_attenuation(account_id: AccountId, attenuate_score: u64) -> bool;
    fn kill_credit(account_id: AccountId) -> bool;
    fn get_credit_score(account_id: AccountId) -> Option<u64>;
}

impl<T: Trait> CreditInterface<T::AccountId> for Module<T> {
    /// init credit score
    fn initilize_credit(account_id: T::AccountId, score: u64) -> bool {
        if UserCredit::<T>::contains_key(account_id.clone()) {
            false
        } else {
            UserCredit::<T>::insert(account_id, score);
            true
        }
    }

    /// update credit score
    fn update_credit(account_id: T::AccountId, score: u64) -> bool {
        if UserCredit::<T>::contains_key(account_id.clone()) {
            UserCredit::<T>::insert(account_id, score);
            true
        } else {
            false
        }
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

    /// credit score attenuation
    /// Return:
    /// true : success
    /// false: failed
    fn credit_attenuation(account_id: T::AccountId, attenuate_score: u64) -> bool {
        if attenuate_score > 10 {
            return false;
        }
        if !UserCredit::<T>::contains_key(account_id.clone()) {
            return false;
        }
        if let Some(score) = UserCredit::<T>::get(account_id.clone()) {
            if score <= CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD {
                return false;
            }
            if score - attenuate_score > CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD {
                UserCredit::<T>::insert(account_id, score - attenuate_score);
                return true;
            } else {
                UserCredit::<T>::insert(account_id, CREDIT_SCORE_ATTENUATION_LOW_THRESHOLD);
                return true;
            }
        }
        false
    }

    /// clear credit info
    fn kill_credit(account_id: T::AccountId) -> bool {
        if UserCredit::<T>::contains_key(account_id.clone()) {
            UserCredit::<T>::remove(account_id);
            return true;
        }
        false
    }

    fn get_credit_score(account_id: T::AccountId) -> Option<u64>{
        Self::get_user_credit(account_id)
    }
}
