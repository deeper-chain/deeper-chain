#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath, Time, Vec};
use frame_support::{decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self, ensure_signed};
use sp_core::sr25519;
use sp_io::crypto::sr25519_verify;
use sp_runtime::traits::Saturating;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Timestamp: Time;
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type Moment<T> = <<T as Trait>::Timestamp as Time>::Moment;

type ChannelOf<T> = Chan<<T as frame_system::Trait>::AccountId, Moment<T>>;

// struct to store the registered Device Informatin
// TODO: use blockNumber instead of timestamp
#[derive(Decode, Encode, Default)]
pub struct Chan<AccountId, Timestamp> {
    sender: AccountId,
    receiver: AccountId,
    nonce: u64,
    opened: Timestamp,
    expiration: Timestamp,
}

// events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Timestamp = Moment<T>,
        Balance = BalanceOf<T>,
    {
        ChannelOpened(AccountId, AccountId, u64, Timestamp, Timestamp),
        ChannelClosed(AccountId, AccountId, Timestamp),
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
          let time = T::Timestamp::now();
          let duration_in_mills = duration * 1000;
          let expiration = time.saturating_add(duration_in_mills.into());
          let chan = ChannelOf::<T>{
              sender: sender.clone(),
              receiver: receiver.clone(),
              nonce: nonce.clone(),
              opened: time.clone(),
              expiration: expiration.clone(),
          };
          Channel::<T>::insert((sender.clone(),receiver.clone()), chan);
          Nonce::<T>::insert((sender.clone(),receiver.clone()),nonce+1);
          //Nonce::<T>::mutate((sender.clone(),receiver.clone()),|v|*v+1);
          Self::deposit_event(RawEvent::ChannelOpened(sender,receiver,nonce, time,expiration));
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
          let time = T::Timestamp::now();
          Self::deposit_event(RawEvent::ChannelClosed(sender, receiver, time));
          Ok(())
      }

      #[weight = 10_000]
      // TODO: instead of transfer from sender, transfer from sender's reserved token
      pub fn claim_payment(origin, sender: T::AccountId, session_id: u32, amount: BalanceOf<T>, signature: Vec<u8>) -> DispatchResult {
          let receiver = ensure_signed(origin)?;
          ensure!(Channel::<T>::contains_key((sender.clone(),receiver.clone())), "Channel not exists");


          // close channel if it expires
          let chan = Channel::<T>::get((sender.clone(),receiver.clone()));
          if chan.expiration < T::Timestamp::now() {
              Self::_close_channel(&sender, &receiver);
              let time = T::Timestamp::now();
              Self::deposit_event(RawEvent::ChannelClosed(sender, receiver, time));
              return Ok(());
          }

          ensure!(!SessionId::<T>::contains_key((sender.clone(),receiver.clone()),session_id), "SessionID already consumed");
          Self::verify_signature(&sender, &receiver, chan.nonce, session_id, amount, &signature)?;

          T::Currency::transfer(&sender, &receiver, amount, AllowDeath)?; // TODO: check what is AllowDeath
          SessionId::<T>::insert((sender.clone(),receiver.clone()), session_id, true); // mark session_id as used
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2_hash() {
        let bob: [u8; 32] = [
            142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54,
            147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
        ];
        let session_id: u32 = 22;
        let nonce: u64 = 5;
        let amount: u128 = 100;
        let mut data = Vec::new();

        let should_be: [u8; 32] = [
            204, 32, 30, 136, 139, 38, 43, 64, 99, 194, 191, 149, 97, 108, 87, 173, 224, 25, 104,
            100, 0, 179, 72, 91, 202, 84, 34, 190, 178, 119, 59, 41,
        ];

        data.extend_from_slice(&bob);
        data.extend_from_slice(&session_id.to_be_bytes());
        data.extend_from_slice(&amount.to_be_bytes());
        let hash = sp_io::hashing::blake2_256(&data);
        assert_eq!(&hash, &should_be);
    }

    #[test]
    fn test_signature() {
        let sig: [u8; 64] = [
            68, 47, 70, 69, 17, 14, 9, 253, 233, 25, 253, 31, 54, 87, 196, 88, 192, 81, 241, 235,
            51, 175, 232, 189, 181, 176, 89, 123, 223, 237, 162, 39, 79, 234, 237, 116, 157, 88,
            19, 64, 224, 90, 66, 80, 4, 202, 207, 153, 220, 159, 142, 118, 210, 8, 25, 102, 159,
            44, 229, 1, 58, 237, 243, 135,
        ];
        assert_eq!(sig.len(), 64);
        let pk: [u8; 32] = [
            212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133,
            88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
        ];
        assert_eq!(pk.len(), 32);
        let msg: [u8; 32] = [
            204, 32, 30, 136, 139, 38, 43, 64, 99, 194, 191, 149, 97, 108, 87, 173, 224, 25, 104,
            100, 0, 179, 72, 91, 202, 84, 34, 190, 178, 119, 59, 41,
        ];

        let pk = sr25519::Public::from_raw(pk);
        let sig = sr25519::Signature::from_slice(&sig);
        println!("pk:{:?}", pk);
        println!("sig:{:?}", sig);
        println!("msg:{:?}", msg);
        let verified = sr25519_verify(&sig, &msg, &pk);
        assert_eq!(verified, true);
    }
}
