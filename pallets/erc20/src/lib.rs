#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Codec, Decode, Encode};
use frame_support::traits::{Vec, Get};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, Parameter,
};
use frame_system::{self, ensure_signed};
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Member};

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type TokenBalance: CheckedAdd
        + CheckedSub
        + Parameter
        + Member
        + Codec
        + Copy
        + AtLeast32BitUnsigned
        + Default;

    type MaxTokenNameLen: Get<usize>;
    type MaxTokenTickerLen: Get<usize>;
}

// struct to store the token details
#[derive(Decode, Encode, Default, Clone, PartialEq, Debug)]
pub struct Erc20Token<T> {
    name: Vec<u8>,
    ticker: Vec<u8>,
    total_supply: T,
}

// error messages
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// invalid token name, too long
        InvalidTokenName,
        /// invalid token ticker, too long
        InvalidTokenTicker,
        /// invalid account
        InvalidAccount,
        /// allowance doesn't exist
        AllowanceNotExist,
        /// allowance isn't enough
        AllowanceNotEnough,
        /// balance isn't enough
        BalanceNotEnough,
        /// overflow in calculating next token id
        TokenIdOverflow,
        /// overflow in calculating balance
        BalanceOverflow,
        /// overflow in calculating allowance
        AllowanceOverflow,
    }
}

// events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = <T as Trait>::TokenBalance,
    {
        // event for transfer of tokens
        // tokenid, from, to, value
        Transfer(u32, AccountId, AccountId, Balance),
        // event when an approval is made
        // tokenid, owner, spender, value
        Approval(u32, AccountId, AccountId, Balance),
    }
);

// storage for this module
decl_storage! {
  trait Store for Module<T: Trait> as Erc20 {
      // token id nonce for storing the next token id available for token initialization
      // inspired by the AssetId in the SRML assets module
      TokenId get(fn token_id): u32;
      // details of the token corresponding to a token id
      Tokens get(fn token_details): map hasher(blake2_128_concat) u32 => Erc20Token<T::TokenBalance>;
      // balances mapping for an account and token
      BalanceOf get(fn balance_of): map hasher(blake2_128_concat) (u32, T::AccountId) => T::TokenBalance;
      // allowance for an account and token
      Allowance get(fn allowance): map hasher(blake2_128_concat) (u32, T::AccountId, T::AccountId) => T::TokenBalance;
  }
}

// public interface for this runtime module
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
      // initialize the default event for this module
      fn deposit_event() = default;

      // initializes a new token
      // generates an integer token_id so that all tokens are unique
      // takes a name, ticker, total supply for the token
      // makes the initiating account the owner of the token
      // the balance of the owner is set to total supply
      #[weight = 10_000]
      fn init(origin, name: Vec<u8>, ticker: Vec<u8>, total_supply: T::TokenBalance) -> DispatchResult {
          let sender = ensure_signed(origin)?;

          // checking max size for name and ticker
          // byte arrays (vecs) with no max size should be avoided
          ensure!(name.len() <= T::MaxTokenNameLen::get(), Error::<T>::InvalidTokenName);
          ensure!(ticker.len() <= T::MaxTokenTickerLen::get(), Error::<T>::InvalidTokenTicker);

          let token_id = Self::token_id();
          let next_token_id = token_id.checked_add(1).ok_or(Error::<T>::TokenIdOverflow)?;
          TokenId::put(next_token_id);

          let token = Erc20Token {
              name,
              ticker,
              total_supply,
          };

          <Tokens<T>>::insert(token_id, token);
          <BalanceOf<T>>::insert((token_id, sender), total_supply);

          Ok(())
      }

      // transfer tokens from one account to another
      // origin is assumed as sender
      #[weight = 10_000]
      fn transfer(_origin, token_id: u32, to: T::AccountId, value: T::TokenBalance) -> DispatchResult {
          let sender = ensure_signed(_origin)?;
          Self::_transfer(token_id, sender, to, value)
      }

      // approve token transfer from one account to another
      // once this is done, transfer_from can be called with corresponding values
      #[weight = 10_000]
      fn approve(_origin, token_id: u32, spender: T::AccountId, value: T::TokenBalance) -> DispatchResult {
          let sender = ensure_signed(_origin)?;
          ensure!(<BalanceOf<T>>::contains_key((token_id, sender.clone())), Error::<T>::InvalidAccount);
          let allowance = Self::allowance((token_id, sender.clone(), spender.clone()));
          let updated_allowance = allowance.checked_add(&value).ok_or(Error::<T>::AllowanceOverflow)?;
          <Allowance<T>>::insert((token_id, sender.clone(), spender.clone()), updated_allowance);

          Self::deposit_event(RawEvent::Approval(token_id, sender.clone(), spender.clone(), value));

          Ok(())
      }

      // the ERC20 standard transfer_from function
      // implemented in the open-zeppelin way - increase/decrease allownace
      // if approved, transfer from an account to another account without owner's signature
      #[weight = 10_000]
      pub fn transfer_from(_origin, token_id: u32, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> DispatchResult {
        ensure!(<Allowance<T>>::contains_key((token_id, from.clone(), to.clone())), Error::<T>::AllowanceNotExist);
        let allowance = Self::allowance((token_id, from.clone(), to.clone()));
        ensure!(allowance >= value, Error::<T>::AllowanceNotEnough);

        // using checked_sub (safe math) to avoid overflow
        let updated_allowance = allowance.checked_sub(&value).ok_or(Error::<T>::AllowanceOverflow)?;
        <Allowance<T>>::insert((token_id, from.clone(), to.clone()), updated_allowance);

        Self::deposit_event(RawEvent::Approval(token_id, from.clone(), to.clone(), value));
        Self::_transfer(token_id, from, to, value)
      }
  }
}

// implementation of mudule
// utility and private functions
// if marked public, accessible by other modules
impl<T: Trait> Module<T> {
    // the ERC20 standard transfer function
    // internal
    fn _transfer(
        token_id: u32,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> DispatchResult {
        ensure!(
            <BalanceOf<T>>::contains_key((token_id, from.clone())),
            Error::<T>::InvalidAccount
        );
        let sender_balance = Self::balance_of((token_id, from.clone()));
        ensure!(sender_balance >= value, Error::<T>::BalanceNotEnough);

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;
        let receiver_balance = Self::balance_of((token_id, to.clone()));
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or(Error::<T>::BalanceOverflow)?;
        // reduce sender's balance
        <BalanceOf<T>>::insert((token_id, from.clone()), updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert((token_id, to.clone()), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(token_id, from, to, value));
        Ok(())
    }
}
