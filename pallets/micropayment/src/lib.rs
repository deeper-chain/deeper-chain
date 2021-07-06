#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
pub use pallet::*;

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

#[frame_support::pallet]
pub mod pallet {
    use frame_support::codec::{Decode, Encode};
    use frame_support::traits::{Currency, Get, Vec};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use log::error;
    use pallet_balances::MutableCurrency;
    use sp_core::sr25519;
    use sp_io::crypto::sr25519_verify;
    use sp_runtime::{traits::Zero, SaturatedConversion};
    extern crate alloc;
    use alloc::collections::btree_map::BTreeMap;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId> + MutableCurrency<Self::AccountId>;
        type DayToBlocknum: Get<u32>;

        /// data traffic to DPR ratio
        #[pallet::constant]
        type DataPerDPR: Get<u64>;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    type ChannelOf<T> = Chan<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
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

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // get channel info
    #[pallet::storage]
    #[pallet::getter(fn get_channel)]
    pub(super) type Channel<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        ChannelOf<T>,
        ValueQuery,
    >;

    // nonce indicates the next available value; increase by one whenever open a new channel for an account pair
    #[pallet::storage]
    #[pallet::getter(fn get_nonce)]
    pub(super) type Nonce<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), u64, ValueQuery>;

    // session id
    #[pallet::storage]
    #[pallet::getter(fn get_session_id)]
    pub(super) type SessionId<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), u32, OptionQuery>;

    // the last block that an ccount has micropayment transaction involved
    #[pallet::storage]
    #[pallet::getter(fn last_updated)]
    pub(super) type LastUpdated<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, ValueQuery>;

    // record total micorpayment channel balance of accountid
    #[pallet::storage]
    #[pallet::getter(fn total_micropayment_chanel_balance)]
    pub(super) type TotalMicropaymentChannelBalance<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    // record server accounts who has claimed micropayment during a given block
    #[pallet::storage]
    #[pallet::getter(fn get_server_by_block)]
    pub(super) type ServerByBlock<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::BlockNumber,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    // record client accumulated payments to a given server account during a given block
    #[pallet::storage]
    #[pallet::getter(fn get_clientpayment_by_block_server)]
    pub(super) type ClientPaymentByBlockServer<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::BlockNumber, T::AccountId),
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
        ValueQuery,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ChannelOpened(
            T::AccountId,
            T::AccountId,
            BalanceOf<T>,
            u64,
            T::BlockNumber,
            T::BlockNumber,
        ),
        ChannelClosed(T::AccountId, T::AccountId, T::BlockNumber),
        ClaimPayment(T::AccountId, T::AccountId, BalanceOf<T>),
        BalanceAdded(T::AccountId, T::AccountId, BalanceOf<T>, T::BlockNumber),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Not enough balance
        NotEnoughBalance,
        /// Micropayment channel not exist
        ChannelNotExist,
        /// channel has been opened
        ChannelAlreadyOpened,
        /// sender can only close expired channel
        UnexpiredChannelCannotBeClosedBySender,
        /// Sender and receiver are the same
        SameChannelEnds,
        /// Session has already been consumed
        SessionError,
        /// Invalid signature, cannot be verified
        InvalidSignature,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        // duration is in units of second
        pub fn open_channel(
            origin: OriginFor<T>,
            receiver: T::AccountId,
            lock_amt: BalanceOf<T>,
            duration: u32,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                !Channel::<T>::contains_key(sender.clone(), receiver.clone()),
                Error::<T>::ChannelAlreadyOpened
            );
            ensure!(
                sender.clone() != receiver.clone(),
                Error::<T>::SameChannelEnds
            );
            let nonce = Nonce::<T>::get((sender.clone(), receiver.clone()));
            let start_block = <frame_system::Module<T>>::block_number();
            let duration_block = (duration as u32) * T::DayToBlocknum::get();
            let expiration = start_block + T::BlockNumber::from(duration_block);
            let chan = ChannelOf::<T> {
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
            Channel::<T>::insert(sender.clone(), receiver.clone(), chan);
            if TotalMicropaymentChannelBalance::<T>::contains_key(&sender) {
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&sender, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = Some(total_balance + lock_amt);
                });
            } else {
                TotalMicropaymentChannelBalance::<T>::insert(sender.clone(), lock_amt);
            }
            Self::deposit_event(Event::ChannelOpened(
                sender,
                receiver,
                lock_amt,
                nonce,
                start_block,
                expiration,
            ));
            Ok(().into())
        }

        // make sure claim your payment before close the channel
        #[pallet::weight(10_000)]
        pub fn close_channel(
            origin: OriginFor<T>,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            // receiver can close channel at any time;
            // sender can only close expired channel.
            let signer = ensure_signed(origin)?;

            if Channel::<T>::contains_key(account_id.clone(), signer.clone()) {
                // signer is receiver
                let chan = Channel::<T>::get(account_id.clone(), signer.clone());
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&account_id, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = if total_balance > chan.balance {
                        Some(total_balance - chan.balance)
                    } else {
                        None
                    };
                });
                Self::deposit_into_account(&account_id, chan.balance);
                Self::_close_channel(&account_id, &signer);
                let end_block = <frame_system::Module<T>>::block_number();
                Self::deposit_event(Event::ChannelClosed(account_id, signer, end_block));
                return Ok(().into());
            } else if Channel::<T>::contains_key(signer.clone(), account_id.clone()) {
                // signer is sender
                let chan = Channel::<T>::get(signer.clone(), account_id.clone());
                let current_block = <frame_system::Module<T>>::block_number();
                if chan.expiration < current_block {
                    TotalMicropaymentChannelBalance::<T>::mutate_exists(&signer, |b| {
                        let total_balance = b.take().unwrap_or_default();
                        *b = if total_balance > chan.balance {
                            Some(total_balance - chan.balance)
                        } else {
                            None
                        };
                    });
                    Self::deposit_into_account(&signer, chan.balance);
                    Self::_close_channel(&signer, &account_id);
                    let end_block = current_block;
                    Self::deposit_event(Event::ChannelClosed(signer, account_id, end_block));
                    return Ok(().into());
                } else {
                    Err(Error::<T>::UnexpiredChannelCannotBeClosedBySender)?
                }
            } else {
                Err(Error::<T>::ChannelNotExist)?
            }
        }

        // sender close all expired channels on chain
        #[pallet::weight(10_000)]
        pub fn close_expired_channels(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            // sender can only close expired channel.
            let sender = ensure_signed(origin)?;
            for (receiver, chan) in Channel::<T>::iter_prefix(sender.clone()) {
                let current_block = <frame_system::Module<T>>::block_number();
                if chan.expiration < current_block {
                    TotalMicropaymentChannelBalance::<T>::mutate_exists(&sender, |b| {
                        let total_balance = b.take().unwrap_or_default();
                        *b = if total_balance > chan.balance {
                            Some(total_balance - chan.balance)
                        } else {
                            None
                        };
                    });
                    Self::deposit_into_account(&sender.clone(), chan.balance);
                    Self::_close_channel(&sender, &receiver);
                    let end_block = current_block;
                    Self::deposit_event(Event::ChannelClosed(sender.clone(), receiver, end_block));
                }
            }
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn add_balance(
            origin: OriginFor<T>,
            receiver: T::AccountId,
            amt: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                Channel::<T>::contains_key(&sender, &receiver),
                Error::<T>::ChannelNotExist
            );
            if !Self::take_from_account(&sender, amt) {
                error!("Not enough free balance to add into channel");
                Err(Error::<T>::NotEnoughBalance)?
            }
            Channel::<T>::mutate(&sender, &receiver, |c| {
                (*c).balance += amt;
            });
            TotalMicropaymentChannelBalance::<T>::mutate_exists(&sender, |b| {
                let total_balance = b.take().unwrap_or_default();
                *b = Some(total_balance + amt);
            });
            let end_block = <frame_system::Module<T>>::block_number();
            Self::deposit_event(Event::BalanceAdded(sender, receiver, amt, end_block));
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        // TODO: instead of transfer from sender, transfer from sender's reserved token
        pub fn claim_payment(
            origin: OriginFor<T>,
            sender: T::AccountId,
            session_id: u32,
            amount: BalanceOf<T>,
            signature: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let receiver = ensure_signed(origin)?;
            ensure!(
                Channel::<T>::contains_key(sender.clone(), receiver.clone()),
                Error::<T>::ChannelNotExist
            );

            // close channel if it expires
            let mut chan = Channel::<T>::get(sender.clone(), receiver.clone());
            let current_block = <frame_system::Module<T>>::block_number();
            if chan.expiration < current_block {
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&sender, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = if total_balance > chan.balance {
                        Some(total_balance - chan.balance)
                    } else {
                        None
                    };
                });
                Self::deposit_into_account(&sender, chan.balance);
                Self::_close_channel(&sender, &receiver);
                let end_block = current_block;
                Self::deposit_event(Event::ChannelClosed(sender, receiver, end_block));
                return Ok(().into());
            }

            if SessionId::<T>::contains_key((sender.clone(), receiver.clone()))
                && session_id
                    != Self::get_session_id((sender.clone(), receiver.clone())).unwrap_or(0) + 1
            {
                Err(Error::<T>::SessionError)?
            }
            Self::verify_signature(
                &sender, &receiver, chan.nonce, session_id, amount, &signature,
            )?;
            SessionId::<T>::insert((sender.clone(), receiver.clone()), session_id); // mark session_id as used

            if chan.balance < amount {
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&sender, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = if total_balance > chan.balance {
                        Some(total_balance - chan.balance)
                    } else {
                        None
                    };
                });
                Self::deposit_into_account(&receiver, chan.balance);
                Self::update_micropayment_information(&sender, &receiver, chan.balance);
                // no balance in channel now, just close it
                Self::_close_channel(&sender, &receiver);
                let end_block = <frame_system::Module<T>>::block_number();
                Self::deposit_event(Event::ChannelClosed(
                    sender.clone(),
                    receiver.clone(),
                    end_block,
                ));
                error!("Channel not enough balance");
                Err(Error::<T>::NotEnoughBalance)?
            }

            chan.balance -= amount;
            Channel::<T>::insert(sender.clone(), receiver.clone(), chan);
            TotalMicropaymentChannelBalance::<T>::mutate_exists(&sender, |b| {
                let total_balance = b.take().unwrap_or_default();
                *b = if total_balance > amount {
                    Some(total_balance - amount)
                } else {
                    None
                };
            });
            Self::deposit_into_account(&receiver, amount);
            Self::update_micropayment_information(&sender, &receiver, amount);
            Self::deposit_event(Event::ClaimPayment(sender, receiver, amount));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn _close_channel(sender: &T::AccountId, receiver: &T::AccountId) {
            // remove all the sesson_ids of given channel
            SessionId::<T>::remove((sender.clone(), receiver.clone()));
            Channel::<T>::remove(sender.clone(), receiver.clone());
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
        ) -> DispatchResultWithPostInfo {
            let mut pk = [0u8; 32];
            pk.copy_from_slice(&sender.encode());
            let pub_key = sr25519::Public::from_raw(pk);

            let mut sig = [0u8; 64];
            sig.copy_from_slice(&signature);
            let sig = sr25519::Signature::from_slice(&sig);

            let msg = Self::construct_byte_array_and_hash(&receiver, nonce, session_id, amount);

            let verified = sr25519_verify(&sig, &msg, &pub_key);
            ensure!(verified, Error::<T>::InvalidSignature);

            Ok(().into())
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

        pub fn update_micropayment_information(
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
                    for (client, bal) in ClientPaymentByBlockServer::<T>::iter_prefix((
                        T::BlockNumber::from(n),
                        &server,
                    )) {
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
            if let Ok(actual_balance) = actual {
                if actual_balance < amt {
                    return false;
                } else {
                    return true;
                }
            }
            false
        }

        fn deposit_into_account(acc: &T::AccountId, amt: BalanceOf<T>) {
            let _ = T::Currency::mutate_account_balance(acc, |account| {
                account.free += amt;
            });
        }
    }
}
