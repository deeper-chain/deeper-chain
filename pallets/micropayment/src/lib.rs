#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::collections::btree_map::BTreeMap;

use frame_support::codec::{Decode, Encode};
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath, Vec};
use frame_support::{decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self, ensure_signed};
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
    type Currency: Currency<Self::AccountId>;
}

// todo: this is import from runtime constant
const MILLISECS_PER_BLOCK: u32 = 5000;
const DAY_TO_BLOCKNUM: u32 = 24 * 3600 * 1000 / MILLISECS_PER_BLOCK;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type ChannelOf<T> =
    Chan<<T as frame_system::Trait>::AccountId, <T as frame_system::Trait>::BlockNumber>;

// struct to store the registered Device Informatin
#[derive(Decode, Encode, Default)]
pub struct Chan<AccountId, BlockNumber> {
    sender: AccountId,
    receiver: AccountId,
    nonce: u64,
    opened: BlockNumber,
    expiration: BlockNumber,
}

// events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        Balance = BalanceOf<T>,
    {
        ChannelOpened(AccountId, AccountId, u64, BlockNumber, BlockNumber),
        ChannelClosed(AccountId, AccountId, BlockNumber),
        ClaimPayment(AccountId, AccountId, Balance),
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
      // initialize the default event for this module
      fn deposit_event() = default;

      #[weight = 10_000]
      // duration is in units of second
      // TODO: reserve enough tokens for micropayment when open the channel
      pub fn open_channel(origin, receiver: T::AccountId, duration: u32) -> DispatchResult {
          let sender = ensure_signed(origin)?;
          ensure!(!Channel::<T>::contains_key((sender.clone(),receiver.clone())), "Channel already opened");
          ensure!(sender.clone() != receiver.clone(), "Channel should connect two different accounts");
          let nonce = Nonce::<T>::get((sender.clone(),receiver.clone()));
          let start_block =  <frame_system::Module<T>>::block_number();
          let duration_block = (duration as u32) * DAY_TO_BLOCKNUM;
          let expiration = start_block + T::BlockNumber::from(duration_block);
          let chan = ChannelOf::<T>{
              sender: sender.clone(),
              receiver: receiver.clone(),
              nonce: nonce.clone(),
              opened: start_block.clone(),
              expiration: expiration.clone(),
          };
          Channel::<T>::insert((sender.clone(),receiver.clone()), chan);
          Nonce::<T>::insert((sender.clone(),receiver.clone()),nonce+1);
          //Nonce::<T>::mutate((sender.clone(),receiver.clone()),|v|*v+1);
          Self::deposit_event(RawEvent::ChannelOpened(sender,receiver,nonce,start_block,expiration));
          Ok(())
      }

      #[weight = 10_000]
      // make sure claim your payment before close the channel
      // TODO: refund the rest of reserved tokens back to sender
      pub fn close_channel(origin, sender: T::AccountId) -> DispatchResult {
          // only receiver can close the channel
          let receiver = ensure_signed(origin)?;
          ensure!(Channel::<T>::contains_key((sender.clone(),receiver.clone())), "Channel not exists");
          Self::_close_channel(&sender, &receiver);
          let end_block =  <frame_system::Module<T>>::block_number();
          Self::deposit_event(RawEvent::ChannelClosed(sender, receiver, end_block));
          Ok(())
      }

      #[weight = 10_000]
      // TODO: instead of transfer from sender, transfer from sender's reserved token
      pub fn claim_payment(origin, sender: T::AccountId, session_id: u32, amount: BalanceOf<T>, signature: Vec<u8>) -> DispatchResult {
          let receiver = ensure_signed(origin)?;
          ensure!(Channel::<T>::contains_key((sender.clone(),receiver.clone())), "Channel not exists");


          // close channel if it expires
          let chan = Channel::<T>::get((sender.clone(),receiver.clone()));
          let current_block = <frame_system::Module<T>>::block_number();
          if chan.expiration < current_block {
              Self::_close_channel(&sender, &receiver);
              let end_block = current_block;
              Self::deposit_event(RawEvent::ChannelClosed(sender, receiver, end_block));
              return Ok(());
          }

          ensure!(!SessionId::<T>::contains_key((sender.clone(),receiver.clone()),session_id), "SessionID already consumed");
          Self::verify_signature(&sender, &receiver, chan.nonce, session_id, amount, &signature)?;

          T::Currency::transfer(&sender, &receiver, amount, AllowDeath)?; // TODO: check what is AllowDeath
          SessionId::<T>::insert((sender.clone(),receiver.clone()), session_id, true); // mark session_id as used

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
        ensure!(verified, "Fail to verify signature");

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
}
