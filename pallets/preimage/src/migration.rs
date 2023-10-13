// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
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

//! Storage migrations for the preimage pallet.

use super::*;
use frame_support::{
    storage_alias,
    traits::{ConstU32, OnRuntimeUpgrade},
};

/// The log target.
const TARGET: &'static str = "runtime::preimage::migration::v1";

/// The original data layout of the preimage pallet without a specific version number.
mod v0 {
    use super::*;

    #[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
    pub enum RequestStatus<AccountId, Balance> {
        Unrequested(Option<(AccountId, Balance)>),
        Requested(u32),
    }

    #[storage_alias]
    pub type PreimageFor<T: Config> = StorageMap<
        Pallet<T>,
        Identity,
        <T as frame_system::Config>::Hash,
        BoundedVec<u8, ConstU32<MAX_SIZE>>,
    >;

    #[storage_alias]
    pub type StatusFor<T: Config> = StorageMap<
        Pallet<T>,
        Identity,
        <T as frame_system::Config>::Hash,
        RequestStatus<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
    >;
}

pub mod v1 {
    use super::*;

    pub struct Migration<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for Migration<T> {
        fn on_runtime_upgrade() -> Weight {
            let weight = T::DbWeight::get().reads(1);
            if StorageVersion::get::<Pallet<T>>() != 0 {
                log::warn!(
                    target: TARGET,
                    "skipping MovePreimagesIntoBuckets: executed on wrong storage version.\
				Expected version 0"
                );
                return weight;
            }

            let mut old_hashes = Vec::new();
            let mut new_hashes = Vec::new();

            for inner_hash in v0::PreimageFor::<T>::iter_keys() {
                old_hashes.push(inner_hash);
            }

            for (inner_hash, _) in crate::PreimageFor::<T>::iter_keys() {
                new_hashes.push(inner_hash);
            }

            log::warn!(
                target: TARGET,
                "old_hashes {:?} new_hashes {:?}",
                old_hashes,
                new_hashes
            );

            let difference: Vec<_> = old_hashes
                .iter()
                .filter(|&x| !new_hashes.contains(x))
                .collect();

            log::warn!(target: TARGET, "difference {:?}", difference);

            for hash in difference {
                v0::PreimageFor::<T>::remove(hash);
                v0::StatusFor::<T>::remove(hash);
            }
            // crate::StatusFor::<T>::remove(hash, status);
            // crate::PreimageFor::<T>::insert(&(hash, len), preimage);
            StorageVersion::new(1).put::<Pallet<T>>();
            weight.saturating_add(T::DbWeight::get().writes(1))
        }
    }
}
