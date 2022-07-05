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
mod testing_utils;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

#[cfg(any(feature = "runtime-benchmarks"))]
use sp_std::prelude::*;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use frame_support::{
        codec::Encode, dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    #[cfg(feature = "runtime-benchmarks")]
    use node_primitives::AccountCreator;
    use node_primitives::{credit::CreditInterface, VerifySignatureInterface};
    use sp_core::crypto::UncheckedFrom;
    use sp_core::sr25519;
    use sp_io::crypto::sr25519_verify;
    use sp_runtime::traits::TrailingZeroInput;
    use sp_std::prelude::Vec;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
        // CreditInterface of credit pallet
        type CreditInterface: CreditInterface<Self::AccountId, BalanceOf<Self>>;
        // Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
        #[cfg(feature = "runtime-benchmarks")]
        type AccountCreator: AccountCreator<Self::AccountId>;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // atmos_nonce indicates the next available value;
    #[pallet::storage]
    #[pallet::getter(fn atmos_nonce)]
    pub(super) type AtmosNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn atmos_accountid)]
    pub(super) type AtmosAccountid<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn tmp_atmos_accountid)]
    pub(super) type TmpAtmosAccountid<T: Config> = StorageValue<_, T::AccountId>;

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AtmosSignatureValid(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        // Invalid signature
        InvalidSignature,
        /// Invalid atomos nonce
        InvalidAtomosNonce,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let tmp_accout_id = Self::tmp_atmos_accountid();
            if tmp_accout_id.is_some() {
                return T::DbWeight::get().reads_writes(1, 0);
            }
            let old_account_id = Self::atmos_accountid();
            if old_account_id.is_none() {
                return T::DbWeight::get().reads_writes(2, 0);
            }
            TmpAtmosAccountid::<T>::put(old_account_id.unwrap());
            T::DbWeight::get().reads_writes(2, 1)
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::add_credit_by_traffic())]
        pub fn add_credit_by_traffic(
            origin: OriginFor<T>,
            nonce: u64,
            signature: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let server = ensure_signed(origin)?;

            let atmos_nonce_of_server = Self::atmos_nonce(&server).unwrap_or_default();
            ensure!(
                nonce == atmos_nonce_of_server,
                Error::<T>::InvalidAtomosNonce
            );

            Self::verify_atomos_signature(nonce, &signature, server.clone())?;
            Self::deposit_event(Event::AtmosSignatureValid(server.clone()));
            AtmosNonce::<T>::insert(&server, atmos_nonce_of_server + 1u64);
            T::CreditInterface::update_credit_by_traffic(server);
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_atmos_pubkey())]
        pub fn set_atmos_pubkey(
            origin: OriginFor<T>,
            pubkey: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            <AtmosAccountid<T>>::put(pubkey);
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_atmos_pubkey())]
        pub fn set_atmos_tmp_pubkey(
            origin: OriginFor<T>,
            pubkey: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            <TmpAtmosAccountid<T>>::put(pubkey);
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn verify_atomos_signature(
            nonce: u64,
            signature: &[u8],
            sender: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let zero_account = T::AccountId::decode(&mut TrailingZeroInput::new(&[][..]))
                .expect("infinite input; qed");
            let atomos_accountid = Self::atmos_accountid().unwrap_or(zero_account.clone());
            let tmp_atomos_accountid = Self::tmp_atmos_accountid().unwrap_or(zero_account);
            match Self::do_verify(nonce, signature, sender.clone(), atomos_accountid) {
                Err(_) => Self::do_verify(nonce, signature, sender, tmp_atomos_accountid),
                Ok(_) => Ok(().into()),
            }
        }

        fn do_verify(
            nonce: u64,
            signature: &[u8],
            sender: T::AccountId,
            atomos_accountid: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let mut pk = [0u8; 32];
            pk.copy_from_slice(&atomos_accountid.encode());

            let pub_key = sr25519::Public::from_raw(pk);
            let sig = sr25519::Signature::from_slice(&signature);
            let mut data = Vec::new();
            data.extend_from_slice(&atomos_accountid.encode());
            data.extend_from_slice(&nonce.to_be_bytes());
            data.extend_from_slice(&sender.encode());
            let msg = sp_io::hashing::blake2_256(&data);

            let verified = sr25519_verify(
                &sig.unwrap_or(UncheckedFrom::unchecked_from([0; 64])),
                &msg,
                &pub_key,
            );
            ensure!(verified, Error::<T>::InvalidSignature);
            Ok(().into())
        }
    }

    impl<T: Config> VerifySignatureInterface<T::AccountId> for Pallet<T> {
        fn verify_atomos_signature(nonce: u64, signature: Vec<u8>, sender: T::AccountId) -> bool {
            Self::verify_atomos_signature(nonce, &signature, sender.clone()).is_ok()
        }
    }
}
