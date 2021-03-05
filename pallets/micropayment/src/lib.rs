#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::collections::btree_map::BTreeMap;

use frame_support::codec::{Decode, Encode};
use frame_support::traits::{Currency, Vec, Get};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self, ensure_signed};
use log::error;
use pallet_balances::MutableCurrency;
use sp_core::sr25519;
use sp_io::crypto::sr25519_verify;
use sp_runtime::traits::Zero;
use sp_runtime::SaturatedConversion;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub(crate) const LOG_TARGET: &'static str = "micropayment";
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
    type Currency: Currency<Self::AccountId> + MutableCurrency<Self::AccountId>;

    type DayToBlocknum: Get<u32>;
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type ChannelOf<T> = Chan<
    <T as frame_system::Trait>::AccountId,
    <T as frame_system::Trait>::BlockNumber,
    BalanceOf<T>,
>;

// struct to store the registered Device Informatin
#[derive(Decode, Encode, Default)]
pub struct Chan<AccountId, BlockNumber, Balance> {
    sender: AccountId,
    receiver: AccountId,
    balance: Balance,
    nonce: u64,
    opened: BlockNumber,
    expiration: BlockNumber,
}

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Not enough balance
        NotEnoughBalance,
        /// Micropayment channel not exist
        ChannelNotExist,
        /// channel has been opened
        ChannelAlreadyOpened,
        /// Sender and receiver are the same
        SameChannelEnds,
        /// Session has already been consumed
        SessionAlreadyConsumed,
        /// Invalid signature, cannot be verified
        InvalidSignature,
    }
}

// events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        Balance = BalanceOf<T>,
    {
        // ChannelOpened(sender,receiver,balance,nonce,openblock,expirationblock)
        ChannelOpened(AccountId, AccountId, Balance, u64, BlockNumber, BlockNumber),
        ChannelClosed(AccountId, AccountId, BlockNumber),
        ClaimPayment(AccountId, AccountId, Balance),
        BalanceAdded(AccountId, AccountId, Balance, BlockNumber),
    }
);

// storage for this module
decl_storage! {
  trait Store for Module<T: Trait> as Device {
      Channel get(fn get_channel): map hasher(blake2_128_concat) (T::AccountId, T::AccountId) => ChannelOf<T>;
      // nonce indicates the next available value; increase by one whenever open a new channel for an account pair
      Nonce get(fn get_nonce): map hasher(blake2_128_concat) (T::AccountId, T::AccountId)  => u64;
      SessionId get(fn get_session_id): double_map hasher(blake2_128_concat) (T::AccountId, T::AccountId), hasher(blake2_128_concat) u32 => bool;
      // the last block that an ccount has micropayment transaction involved
      LastUpdated get(fn last_updated): map hasher(blake2_128_concat) T::AccountId => T::BlockNumber;
      // record server accounts who has claimed micropayment during a given block
      ServerByBlock get(fn get_server_by_block): double_map hasher(blake2_128_concat) T::BlockNumber, hasher(blake2_128_concat) T::AccountId => bool;
      // record client accumulated payments to a given server account during a given block
      ClientPaymentByBlockServer get(fn get_clientpayment_by_block_server): double_map hasher(blake2_128_concat) (T::BlockNumber, T::AccountId), hasher(blake2_128_concat) T::AccountId => BalanceOf<T>;
  }

}

// public interface for this runtime module
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
      // Errors must be initialized if they are used by the pallet.
      type Error = Error<T>;
      // initialize the default event for this module
      fn deposit_event() = default;

      #[weight = 10_000]
      // duration is in units of second
      pub fn open_channel(origin, receiver: T::AccountId, lock_amt: BalanceOf<T>, duration: u32) -> DispatchResult {
          let sender = ensure_signed(origin)?;
          ensure!(!Channel::<T>::contains_key((sender.clone(),receiver.clone())), Error::<T>::ChannelAlreadyOpened);
          ensure!(sender.clone() != receiver.clone(), Error::<T>::SameChannelEnds);
          let nonce = Nonce::<T>::get((sender.clone(),receiver.clone()));
          let start_block =  <frame_system::Module<T>>::block_number();
          let duration_block = (duration as u32) * T::DayToBlocknum::get();
          let expiration = start_block + T::BlockNumber::from(duration_block);
          let chan = ChannelOf::<T>{
              sender: sender.clone(),
              receiver: receiver.clone(),
              balance: lock_amt,
              nonce: nonce.clone(),
              opened: start_block.clone(),
              expiration: expiration.clone(),
          };
          if !Self::take_from_account(&sender, lock_amt) {
               error!("Not enough free balance to open channel");
               Err(Error::<T>::NotEnoughBalance)?
          }
          Channel::<T>::insert((sender.clone(),receiver.clone()), chan);
          Self::deposit_event(RawEvent::ChannelOpened(sender,receiver,lock_amt,nonce,start_block,expiration));
          Ok(())
      }

      #[weight = 10_000]
      // make sure claim your payment before close the channel
      pub fn close_channel(origin, sender: T::AccountId) -> DispatchResult {
          // only receiver can close the channel
          let receiver = ensure_signed(origin)?;
          ensure!(Channel::<T>::contains_key((sender.clone(),receiver.clone())), Error::<T>::ChannelNotExist);
          let chan = Channel::<T>::get((sender.clone(),receiver.clone()));
          Self::deposit_into_account(&sender, chan.balance);
          Self::_close_channel(&sender, &receiver);
          let end_block =  <frame_system::Module<T>>::block_number();
          Self::deposit_event(RawEvent::ChannelClosed(sender, receiver, end_block));
          Ok(())
      }

      #[weight = 10_000]
      pub fn add_balance(origin, receiver: T::AccountId, amt: BalanceOf<T>) -> DispatchResult {
          let sender = ensure_signed(origin)?;
          ensure!(Channel::<T>::contains_key((&sender, &receiver)), Error::<T>::ChannelNotExist);
          if !Self::take_from_account(&sender, amt) {
               error!("Not enough free balance to add into channel");
               Err(Error::<T>::NotEnoughBalance)?
          }
          Channel::<T>::mutate((&sender, &receiver),|c|{
              (*c).balance += amt;
          });
          let end_block = <frame_system::Module<T>>::block_number();
          Self::deposit_event(RawEvent::BalanceAdded(sender, receiver, amt, end_block));
          Ok(())
      }

      #[weight = 10_000]
      // TODO: instead of transfer from sender, transfer from sender's reserved token
      pub fn claim_payment(origin, sender: T::AccountId, session_id: u32, amount: BalanceOf<T>, signature: Vec<u8>) -> DispatchResult {
          let receiver = ensure_signed(origin)?;
          ensure!(Channel::<T>::contains_key((sender.clone(),receiver.clone())), Error::<T>::ChannelNotExist);

          // close channel if it expires
          let mut chan = Channel::<T>::get((sender.clone(),receiver.clone()));
          let current_block = <frame_system::Module<T>>::block_number();
          if chan.expiration < current_block {
              Self::_close_channel(&sender, &receiver);
              let end_block = current_block;
              Self::deposit_event(RawEvent::ChannelClosed(sender, receiver, end_block));
              return Ok(());
          }

          ensure!(!SessionId::<T>::contains_key((sender.clone(),receiver.clone()),session_id), Error::<T>::SessionAlreadyConsumed);
          Self::verify_signature(&sender, &receiver, chan.nonce, session_id, amount, &signature)?;
          SessionId::<T>::insert((sender.clone(),receiver.clone()), session_id, true); // mark session_id as used

          if chan.balance < amount {
               Self::deposit_into_account(&receiver, chan.balance);
               // no balance in channel now, just close it
               Self::_close_channel(&sender, &receiver);
               let end_block =  <frame_system::Module<T>>::block_number();
               Self::deposit_event(RawEvent::ChannelClosed(sender.clone(), receiver.clone(), end_block));
               error!("Channel not enough balance");
               Err(Error::<T>::NotEnoughBalance)?
          }

          chan.balance -= amount;
          Channel::<T>::insert((sender.clone(),receiver.clone()), chan);
          Self::deposit_into_account(&receiver, amount);
          Self::update_micropayment_information(&sender, &receiver, amount);
          Self::deposit_event(RawEvent::ClaimPayment(sender, receiver, amount));
          Ok(())
      }

  }
}

impl<T: Trait> Module<T> {
    fn _close_channel(sender: &T::AccountId, receiver: &T::AccountId) {
        // remove all the sesson_ids of given channel
        SessionId::<T>::remove_prefix((sender.clone(), receiver.clone()));
        Channel::<T>::remove((sender.clone(), receiver.clone()));
        Nonce::<T>::mutate((sender.clone(), receiver.clone()), |v| *v += 1);
    }

    // verify signature, signature is on hash of |receiver_addr|nonce|session_id|amount|
    // during one session_id, a sender can send multiple accumulated
    // micropayments with the same session_id; the receiver can only claim one payment of the same
    // session_id, i.e. the latest accumulated micropayment.
    pub fn verify_signature(
        sender: &T::AccountId,
        receiver: &T::AccountId,
        nonce: u64,
        session_id: u32,
        amount: BalanceOf<T>,
        signature: &Vec<u8>,
    ) -> DispatchResult {
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&sender.encode());
        let pub_key = sr25519::Public::from_raw(pk);

        let mut sig = [0u8; 64];
        sig.copy_from_slice(&signature);
        let sig = sr25519::Signature::from_slice(&sig);

        let msg = Self::construct_byte_array_and_hash(&receiver, nonce, session_id, amount);

        let verified = sr25519_verify(&sig, &msg, &pub_key);
        ensure!(verified, Error::<T>::InvalidSignature);

        Ok(())
    }

    // construct data from |receiver_addr|session_id|amount| and hash it
    fn construct_byte_array_and_hash(
        address: &T::AccountId,
        nonce: u64,
        session_id: u32,
        amount: BalanceOf<T>,
    ) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&address.encode());
        data.extend_from_slice(&nonce.to_be_bytes());
        data.extend_from_slice(&session_id.to_be_bytes());
        data.extend_from_slice(&amount.encode());
        let hash = sp_io::hashing::blake2_256(&data);
        hash
    }

    fn update_micropayment_information(
        sender: &T::AccountId,
        receiver: &T::AccountId,
        amount: BalanceOf<T>,
    ) {
        // update last block
        let block_number = <frame_system::Module<T>>::block_number();
        LastUpdated::<T>::insert(sender.clone(), block_number);
        LastUpdated::<T>::insert(receiver.clone(), block_number);
        log!(
            info,
            "lastupdated block is {:?} for accounts: {:?}, {:?}",
            block_number,
            &sender,
            &receiver
        );
        // update micropaymentinfo
        ServerByBlock::<T>::insert(block_number, receiver.clone(), true);
        let balance = ClientPaymentByBlockServer::<T>::get((&block_number, &receiver), &sender);
        ClientPaymentByBlockServer::<T>::insert(
            (block_number, receiver.clone()),
            sender.clone(),
            balance + amount,
        );

        log!(info, "micropayment info updated at block {:?} for receiver:{:?}, sender:{:?}, with old balance {:?}, new balance {:?}",
                 block_number, &receiver, &sender, balance, balance+amount);
    }

    // calculate accumulated micropayments statitics between block number "from" and "to" inclusively
    // return is a list of (server_account, accumulated_micropayments,
    // num_of_clients) between block "from" and "to" (inclusive)
    pub fn micropayment_statistics(
        from: T::BlockNumber,
        to: T::BlockNumber,
    ) -> Vec<(T::AccountId, BalanceOf<T>, u32)> {
        let mut stats: BTreeMap<T::AccountId, BTreeMap<T::AccountId, BalanceOf<T>>> =
            BTreeMap::new();
        for n in from.saturated_into::<u32>()..(to.saturated_into::<u32>() + 1u32) {
            for (server, _) in ServerByBlock::<T>::iter_prefix(T::BlockNumber::from(n)) {
                for (client, bal) in
                    ClientPaymentByBlockServer::<T>::iter_prefix((T::BlockNumber::from(n), &server))
                {
                    if !stats.contains_key(&server) {
                        let empty: BTreeMap<T::AccountId, BalanceOf<T>> = BTreeMap::new();
                        stats.insert(server.clone(), empty);
                    }
                    let client_balance = stats.get_mut(&server).unwrap();
                    if let Some(b) = client_balance.get_mut(&client) {
                        *b = *b + bal;
                    } else {
                        client_balance.insert(client.clone(), bal);
                    }
                }
            }
        }
        let mut res: Vec<(T::AccountId, BalanceOf<T>, u32)> = Vec::new();
        for (k, v) in stats.iter() {
            let mut counter: u32 = 0;
            let mut total_bal = BalanceOf::<T>::zero();
            for (_, bal) in v.iter() {
                total_bal = total_bal + *bal;
                counter += 1;
            }
            res.push((k.clone(), total_bal, counter));
        }
        res
    }

    // return the last blocknumber for an account join micropayment activity
    pub fn last_update_block(acc: T::AccountId) -> T::BlockNumber {
        LastUpdated::<T>::get(acc)
    }

    // TODO: take ExistentialDeposit into account
    fn take_from_account(acc: &T::AccountId, amt: BalanceOf<T>) -> bool {
        let actual = T::Currency::mutate_account_balance(acc, |account| {
            if amt > account.free {
                return Zero::zero();
            } else {
                account.free -= amt;
            }
            return amt;
        });
        if actual < amt {
            return false;
        }
        true
    }

    fn deposit_into_account(acc: &T::AccountId, amt: BalanceOf<T>) {
        T::Currency::mutate_account_balance(acc, |account| {
            account.free += amt;
        });
    }
}
