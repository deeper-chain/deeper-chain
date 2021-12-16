// Copyright (C) 2021 Deeper Network Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Micropayment pallet for deeper chain
//!
//! This pallet provides functions for Deeper Connect devices to get rewarded
//! for sharing bandwidth. The rewards include payment in DPR tokens and
//! credit accumulation.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod testing_utils;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
#[cfg(any(feature = "runtime-benchmarks"))]
use sp_std::prelude::*;

pub mod weights;

/// This is for benchmarking and testing.
/// Benchmarking requires AccountId32, while testing requires u64 as AccountId.
pub trait AccountCreator<AccountId> {
    fn create_account(string: &'static str) -> AccountId;
}

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use crate::AccountCreator;
    use frame_support::codec::{Decode, Encode};
    use frame_support::traits::{Currency, Get};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use log::error;
    use pallet_balances::MutableCurrency;
    use pallet_credit::CreditInterface;
    use pallet_deeper_node::NodeInterface;
    use sp_core::sr25519;
    use sp_io::crypto::sr25519_verify;
    use sp_runtime::traits::{Saturating, Zero};
    use sp_runtime::DispatchError;
    use sp_std::prelude::Vec;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId> + MutableCurrency<Self::AccountId>;
        type SecsPerBlock: Get<u32>;
        // CreditInterface of credit pallet
        type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;
        // data traffic to DPR ratio
        #[pallet::constant]
        type DataPerDPR: Get<u64>;
        // Create Account trait for benchmarking
        type AccountCreator: AccountCreator<Self::AccountId>;
        // Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
        /// NodeInterface of deeper-node pallet
        type NodeInterface: NodeInterface<Self::AccountId, Self::BlockNumber>;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub type ChannelOf<T> = Chan<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
        BalanceOf<T>,
    >;

    // struct to store micro-payment channel
    #[derive(Decode, Encode, Default, Eq, PartialEq, Debug, scale_info::TypeInfo)]
    pub struct Chan<AccountId, BlockNumber, Balance> {
        pub client: AccountId,
        pub server: AccountId,
        pub balance: Balance,
        pub nonce: u64,
        pub opened: BlockNumber,
        pub expiration: BlockNumber,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // get channel info
    #[pallet::storage]
    #[pallet::getter(fn channel)]
    pub(super) type Channel<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        ChannelOf<T>,
        ValueQuery,
    >;

    // nonce indicates the next available value;
    // increase by one whenever open a new channel for an account pair
    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub(super) type Nonce<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), u64, ValueQuery>;

    // session id
    #[pallet::storage]
    #[pallet::getter(fn session_id)]
    pub(super) type SessionId<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), u32, OptionQuery>;

    // record total micropayment channel balance of accountId
    #[pallet::storage]
    #[pallet::getter(fn total_micropayment_chanel_balance)]
    pub(super) type TotalMicropaymentChannelBalance<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId", T::BlockNumber = "BlockNumber")]
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
        // Not enough balance
        NotEnoughBalance,
        // micropayment channel not exist
        ChannelNotExist,
        // channel has already been opened
        ChannelAlreadyOpened,
        // client can only close expired channel
        UnexpiredChannelCannotBeClosedBySender,
        // Client and server are the same
        SameChannelEnds,
        // Session has already been consumed
        SessionError,
        // Invalid signature
        InvalidSignature,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::open_channel())]

        /// Client opens a channel to the server by locking DPR tokens inside the channel.
        /// The channel is live for a time span specified in seconds.
        pub fn open_channel(
            origin: OriginFor<T>,
            server: T::AccountId,
            lock_amount: BalanceOf<T>,
            duration: u32, // duration is in units of seconds
        ) -> DispatchResultWithPostInfo {
            let client = ensure_signed(origin)?;
            ensure!(
                !Channel::<T>::contains_key(&client, &server),
                Error::<T>::ChannelAlreadyOpened
            );
            ensure!(client != server, Error::<T>::SameChannelEnds);
            let nonce = Nonce::<T>::get((&client, &server));
            let start_block = <frame_system::Pallet<T>>::block_number();
            let duration_blocks = duration / T::SecsPerBlock::get();
            let expiration = start_block + T::BlockNumber::from(duration_blocks);
            let chan = ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: lock_amount,
                nonce: nonce.clone(),
                opened: start_block.clone(),
                expiration: expiration.clone(),
            };
            if !Self::take_from_account(&client, lock_amount) {
                error!("Not enough free balance to open channel");
                Err(Error::<T>::NotEnoughBalance)?
            }
            Channel::<T>::insert(&client, &server, chan);
            if TotalMicropaymentChannelBalance::<T>::contains_key(&client) {
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&client, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = Some(total_balance + lock_amount);
                });
            } else {
                TotalMicropaymentChannelBalance::<T>::insert(&client, lock_amount);
            }
            Self::deposit_event(Event::ChannelOpened(
                client,
                server,
                lock_amount,
                nonce,
                start_block,
                expiration,
            ));
            Ok(().into())
        }

        /// Close the channel and settle the payment
        #[pallet::weight(T::WeightInfo::close_channel())]
        pub fn close_channel(
            origin: OriginFor<T>,
            account_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            // server can close channel at any time;
            // client can only close expired channel.
            let signer = ensure_signed(origin)?;

            if Channel::<T>::contains_key(&account_id, &signer) {
                // signer is server
                let chan = Channel::<T>::get(&account_id, &signer);
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&account_id, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = if total_balance > chan.balance {
                        Some(total_balance - chan.balance)
                    } else {
                        None
                    };
                });
                // return the remaining balance in the channel to the client
                Self::deposit_into_account(&account_id, chan.balance)?;
                Self::_close_channel(&account_id, &signer);
                let end_block = <frame_system::Pallet<T>>::block_number();
                Self::deposit_event(Event::ChannelClosed(account_id, signer, end_block));
                return Ok(().into());
            } else if Channel::<T>::contains_key(&signer, &account_id) {
                // signer is client
                let chan = Channel::<T>::get(&signer, &account_id);

                let current_block = <frame_system::Pallet<T>>::block_number();
                if chan.expiration < current_block
                    || T::NodeInterface::get_eras_offline(&chan.server) >= 1
                {
                    TotalMicropaymentChannelBalance::<T>::mutate_exists(&signer, |b| {
                        let total_balance = b.take().unwrap_or_default();
                        *b = if total_balance > chan.balance {
                            Some(total_balance - chan.balance)
                        } else {
                            None
                        };
                    });
                    // return the remaining balance in the channel to the client
                    Self::deposit_into_account(&signer, chan.balance)?;
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

        /// client close all expired channels.
        /// client can only close expired channel.
        #[pallet::weight(T::WeightInfo::close_expired_channels())]
        pub fn close_expired_channels(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let client = ensure_signed(origin)?;
            for (server, chan) in Channel::<T>::iter_prefix(&client) {
                let current_block = <frame_system::Pallet<T>>::block_number();
                if chan.expiration < current_block {
                    TotalMicropaymentChannelBalance::<T>::mutate_exists(&client, |b| {
                        let total_balance = b.take().unwrap_or_default();
                        *b = if total_balance > chan.balance {
                            Some(total_balance - chan.balance)
                        } else {
                            None
                        };
                    });
                    // return the remaining balance in the channel to the client
                    Self::deposit_into_account(&client, chan.balance)?;
                    Self::_close_channel(&client, &server);
                    let end_block = current_block;
                    Self::deposit_event(Event::ChannelClosed(client.clone(), server, end_block));
                }
            }
            Ok(().into())
        }

        /// Client adds more DPR tokens to the existing channel
        #[pallet::weight(T::WeightInfo::add_balance())]
        pub fn add_balance(
            origin: OriginFor<T>,
            server: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let client = ensure_signed(origin)?;
            ensure!(
                Channel::<T>::contains_key(&client, &server),
                Error::<T>::ChannelNotExist
            );
            if !Self::take_from_account(&client, amount) {
                error!("Not enough free balance to add into channel");
                Err(Error::<T>::NotEnoughBalance)?
            }
            Channel::<T>::mutate(&client, &server, |c| {
                (*c).balance += amount;
            });
            TotalMicropaymentChannelBalance::<T>::mutate_exists(&client, |b| {
                let total_balance = b.take().unwrap_or_default();
                *b = Some(total_balance + amount);
            });
            let end_block = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::BalanceAdded(client, server, amount, end_block));
            Ok(().into())
        }

        /// Server claims payment from the channel
        #[pallet::weight(T::WeightInfo::claim_payment())]
        pub fn claim_payment(
            origin: OriginFor<T>,
            client: T::AccountId,
            session_id: u32,
            amount: BalanceOf<T>,
            signature: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let server = ensure_signed(origin)?;
            ensure!(
                Channel::<T>::contains_key(&client, &server),
                Error::<T>::ChannelNotExist
            );

            // close channel if it expires
            let mut chan = Channel::<T>::get(&client, &server);
            let current_block = <frame_system::Pallet<T>>::block_number();
            if chan.expiration < current_block {
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&client, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = if total_balance > chan.balance {
                        Some(total_balance - chan.balance)
                    } else {
                        None
                    };
                });
                // return the remaining balance in the channel to the client
                Self::deposit_into_account(&client, chan.balance)?;
                Self::_close_channel(&client, &server);
                let end_block = current_block;
                Self::deposit_event(Event::ChannelClosed(client, server, end_block));
                return Ok(().into());
            }

            if SessionId::<T>::contains_key((&client, &server))
                && session_id != Self::session_id((&client, &server)).unwrap_or(0) + 1
            {
                Err(Error::<T>::SessionError)?
            }
            Self::verify_signature(&client, &server, chan.nonce, session_id, amount, &signature)?;
            SessionId::<T>::insert((&client, &server), session_id); // mark session_id as used

            // if there is not enough balance in the channel
            if chan.balance < amount {
                TotalMicropaymentChannelBalance::<T>::mutate_exists(&client, |b| {
                    let total_balance = b.take().unwrap_or_default();
                    *b = if total_balance > chan.balance {
                        Some(total_balance - chan.balance)
                    } else {
                        None
                    };
                });
                // deposit all the balance in the channel to the server's account
                Self::deposit_into_account(&server, chan.balance)?;
                // update server's credit TODO: reuse in future
                //T::CreditInterface::update_credit((server.clone(), chan.balance));
                // no balance in channel now, just close it
                Self::_close_channel(&client, &server);
                let end_block = <frame_system::Pallet<T>>::block_number();
                Self::deposit_event(Event::ChannelClosed(
                    client.clone(),
                    server.clone(),
                    end_block,
                ));
                error!("Channel not enough balance");
                Err(Error::<T>::NotEnoughBalance)?
            }

            chan.balance -= amount;
            Channel::<T>::insert(&client, &server, chan);
            TotalMicropaymentChannelBalance::<T>::mutate_exists(&client, |b| {
                let total_balance = b.take().unwrap_or_default();
                *b = if total_balance > amount {
                    Some(total_balance - amount)
                } else {
                    None
                };
            });
            // deposit the claimed amount to the server's account
            Self::deposit_into_account(&server, amount)?;
            // update server's credit TODO: reuse in future
            //T::CreditInterface::update_credit((server.clone(), amount));
            Self::deposit_event(Event::ClaimPayment(client, server, amount));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Close the channel between the client and server
        fn _close_channel(client: &T::AccountId, server: &T::AccountId) {
            // remove all the session_ids of given channel
            SessionId::<T>::remove((client, server));
            // remove the channel
            Channel::<T>::remove(client, server);
            // increment the nonce
            Nonce::<T>::mutate((client, server), |v| *v += 1);
        }

        /// verify signature, signature is on hash of |server_addr|nonce|session_id|amount|
        /// during one session_id, a client can send multiple accumulated
        /// micropayments with the same session_id; the server can only claim one payment
        /// of the same session_id, i.e. the latest accumulated micropayment.
        pub fn verify_signature(
            client: &T::AccountId,
            server: &T::AccountId,
            nonce: u64,
            session_id: u32,
            amount: BalanceOf<T>,
            signature: &Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let mut pk = [0u8; 32];
            pk.copy_from_slice(&client.encode());
            let pub_key = sr25519::Public::from_raw(pk);

            let mut sig = [0u8; 64];
            sig.copy_from_slice(&signature);
            let sig = sr25519::Signature::from_slice(&sig);

            let msg = Self::construct_byte_array_and_hash(server, nonce, session_id, amount);

            let verified = sr25519_verify(&sig, &msg, &pub_key);
            ensure!(verified, Error::<T>::InvalidSignature);

            Ok(().into())
        }

        // construct data from |server_addr|session_id|amount| and hash it
        pub fn construct_byte_array_and_hash(
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

        /// Deduct the amount from the account free balance
        fn take_from_account(account: &T::AccountId, amount: BalanceOf<T>) -> bool {
            let result = T::Currency::mutate_account_balance(account, |balance| {
                let min_balance = T::Currency::minimum_balance();
                // ensure after taking amount from account, remaining balance is grater than min_balance
                if balance.free <= amount.saturating_add(min_balance) {
                    return Zero::zero();
                } else {
                    balance.free -= amount;
                }
                return amount;
            });
            match result {
                Ok(actual_amount) => actual_amount == amount,
                _ => false,
            }
        }

        /// Deposit the amount to the account free balance
        fn deposit_into_account(
            account: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            T::Currency::mutate_account_balance(account, |balance| {
                balance.free += amount;
            })
        }
    }
}
