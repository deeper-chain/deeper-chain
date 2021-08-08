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
use crate::testing_utils::*;
use crate::Module as Micropayment;
pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::Module as System;
use frame_system::RawOrigin;
use hex_literal::hex;
use sp_std::vec;

/// Grab a funded user with balance_factor DPR.
pub fn create_funded_user<T: Config>(string: &'static str, balance_factor: u32) -> T::AccountId {
    let user = T::AccountCreator::create_account(string);
    let balance = T::Currency::minimum_balance() * balance_factor.into();
    T::Currency::make_free_balance_be(&user, balance);
    T::Currency::issue(balance);
    user
}

benchmarks! {
    open_channel {
        let client = create_funded_user::<T>("Alice", 100);
        let server = create_funded_user::<T>("Bob", 100);
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
        let client = create_funded_user::<T>("Alice", 100);
        let server = create_funded_user::<T>("Bob", 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 0u32.into(),
                expiration: 720u32.into()
            }
        );
    }: _(RawOrigin::Signed(server.clone()), client.clone())
    verify {
        assert!(!Channel::<T>::contains_key(client, server));
    }

    close_expired_channels {
        let client = create_funded_user::<T>("Alice", 100);
        let server = create_funded_user::<T>("Bob", 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 0u32.into(),
                expiration: 720u32.into()
            }
        );
        System::<T>::set_block_number(722u32.into());
    }: _(RawOrigin::Signed(client.clone()))
    verify {
        assert!(!Channel::<T>::contains_key(client, server));
    }

    add_balance {
        let client = create_funded_user::<T>("Alice", 100);
        let server = create_funded_user::<T>("Bob", 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 0u32.into(),
                expiration: 720u32.into()
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
                opened: 0u32.into(),
                expiration: 720u32.into()
            }
        );
    }

    claim_payment {
        let client = create_funded_user::<T>("Alice", 100);
        let server = create_funded_user::<T>("Bob", 100);
        let amount = T::Currency::minimum_balance() * 30u32.into();

        Micropayment::<T>::open_channel(RawOrigin::Signed(client.clone()).into(), server.clone(), amount, 3600)?;
        assert_eq!(
            Micropayment::<T>::channel(&client, &server),
            ChannelOf::<T> {
                client: client.clone(),
                server: server.clone(),
                balance: amount,
                nonce: 0,
                opened: 0u32.into(),
                expiration: 720u32.into()
            }
        );

        let session_id: u32 = 1;
        let nonce: u64 = 0;
        let claim_amount = T::Currency::minimum_balance() * 10u32.into();
        let msg = Micropayment::<T>::construct_byte_array_and_hash(&server, nonce, session_id, amount);
        let signature: [u8; 64] = hex!("eac2d8d9bc0c1a8b9b909fd42280a2687081ad02f5669e0efc0532073b90e1776252b8035bdb94863aae0cc99f507c69a12e5028387d33421cdbe4d7a9edcd85");
    }: _(RawOrigin::Signed(server.clone()), client.clone(), session_id, amount, signature.into())
    verify {
        // TODO
    }
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
