// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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

//! Staking pallet benchmarking.

use super::*;
use crate::pallet::ChannelOf;
use crate::Module as Micropayment;
pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::Module as System;
use frame_system::RawOrigin;
use sp_core::sr25519;
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_runtime::{MultiSignature, MultiSigner};
use sp_std::{convert::TryFrom, vec};

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;

/// Grab a funded user with balance_factor DPR.
pub fn create_funded_user<T: Config>(
    string: &'static str,
    n: u32,
    balance_factor: u32,
) -> T::AccountId {
    let user = account(string, n, SEED);
    let balance = T::Currency::minimum_balance() * balance_factor.into();
    T::Currency::make_free_balance_be(&user, balance);
    T::Currency::issue(balance);
    user
}

pub fn create_sr25519_pubkey(seed: Vec<u8>) -> MultiSigner {
    sr25519_generate(0.into(), Some(seed)).into()
}

pub fn create_sr25519_signature(payload: &[u8], pubkey: MultiSigner) -> MultiSignature {
    let srpubkey = sr25519::Public::try_from(pubkey).unwrap();
    let srsig = sr25519_sign(0.into(), &srpubkey, payload).unwrap();
    srsig.into()
}

/*
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}*/

benchmarks! {
    open_channel {
        let client = create_funded_user::<T>("user",USER_SEED, 100);
        let server = create_funded_user::<T>("user",USER_SEED+1, 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();
    }: _(RawOrigin::Signed(client.clone()), server.clone(), amount, 3600)
    verify {
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client,
                server: server,
                balance: amount,
                nonce: 0,
                opened: 1u32.into(),
                expiration: 721u32.into()
            }
        );
    }

    close_channel {
        let client = create_funded_user::<T>("user",USER_SEED, 100);
        let server = create_funded_user::<T>("user",USER_SEED+1, 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 1u32.into(),
                expiration: 721u32.into()
            }
        );
    }: _(RawOrigin::Signed(server.clone()), client.clone())
    verify {
        assert!(!Channel::<T>::contains_key(client, server));
    }

    close_expired_channels {
        let client = create_funded_user::<T>("user",USER_SEED, 100);
        let server = create_funded_user::<T>("user",USER_SEED+1, 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 1u32.into(),
                expiration: 721u32.into()
            }
        );
        System::<T>::set_block_number(722u32.into());
    }: _(RawOrigin::Signed(client.clone()))
    verify {
        assert!(!Channel::<T>::contains_key(client, server));
    }

    add_balance {
        let client = create_funded_user::<T>("user",USER_SEED, 100);
        let server = create_funded_user::<T>("user",USER_SEED+1, 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 1u32.into(),
                expiration: 721u32.into()
            }
        );

        let add_amount = T::Currency::minimum_balance() * 20u32.into();
    }: _(RawOrigin::Signed(client.clone()), server.clone(), add_amount)
    verify {
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount + add_amount,
                nonce: 0,
                opened: 1u32.into(),
                expiration: 721u32.into()
            }
        );
    }
    /*
    claim_payment {
        let client = create_funded_user::<T>("user",USER_SEED, 100);
        let server = create_funded_user::<T>("user",USER_SEED+1, 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: server.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 1u32.into(),
                expiration: 721u32.into()
            }
        );

        let session_id: u32 = 1;
        let nonce: u64 = 0;
        let claim_amount = T::Currency::minimum_balance() * 10u32.into();

        let mut data = Vec::new();
        data.extend_from_slice(server.pub_key());
        data.extend_from_slice(&nonce.to_be_bytes());
        data.extend_from_slice(&session_id.to_be_bytes());
        data.extend_from_slice(&amount.to_be_bytes());
        let hash = sp_io::hashing::blake2_256(&data);
        let client_pair = create_sr25519_pubkey(b"//client".to_vec());
        let signature = create_sr25519_signature(hash, client_pair);
    }: _(RawOrigin::Signed(server.clone()), client.clone(), session_id, amount, signature)
    verify {

    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_open_channel::<Test>());
            assert_ok!(test_benchmark_close_channel::<Test>());
            assert_ok!(test_benchmark_close_expired_channels::<Test>());
            assert_ok!(test_benchmark_add_balance::<Test>());
            assert_ok!(test_benchmark_claim_payment::<Test>());
        });
    }
}
