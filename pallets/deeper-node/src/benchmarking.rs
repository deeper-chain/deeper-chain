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
use crate::Pallet as DeeperNode;
use core::str::FromStr;
pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;
use node_primitives::AccountCreator;
use sp_std::vec;

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

benchmarks! {
    where_clause { where T: Config, T: pallet_credit_accumulation::Config }
    register_device {
        DeeperNode::<T>::setup_region_map();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
    }: _(RawOrigin::Signed(user.clone()), vec![1, 2, 3, 4], "US".as_bytes().to_vec())
    verify {
        let node = DeeperNode::<T>::device_info(user);
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());
    }

    unregister_device {
        DeeperNode::<T>::setup_region_map();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        DeeperNode::<T>::register_device(RawOrigin::Signed(user.clone()).into(), vec![1, 2, 3, 4], "US".as_bytes().to_vec())?;
        let node = DeeperNode::<T>::device_info(user.clone());
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());
    }: _(RawOrigin::Signed(user.clone()))
    verify {
    }

    register_server {
        DeeperNode::<T>::setup_region_map();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        DeeperNode::<T>::register_device(RawOrigin::Signed(user.clone()).into(), vec![1, 2, 3, 4], "US".as_bytes().to_vec())?;
        let node = DeeperNode::<T>::device_info(user.clone());
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());
    }: _(RawOrigin::Signed(user.clone()), 1)
    verify {
        let servers = DeeperNode::<T>::servers_by_country("US".as_bytes().to_vec());
        let index = servers.iter().position(|x| *x == user);
        assert_eq!(index, Some(0));
    }

    update_server {
        DeeperNode::<T>::setup_region_map();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        DeeperNode::<T>::register_device(RawOrigin::Signed(user.clone()).into(), vec![1, 2, 3, 4], "US".as_bytes().to_vec())?;
        let node = DeeperNode::<T>::device_info(user.clone());
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());
    }: _(RawOrigin::Signed(user.clone()), 1)
    verify {
    }

    unregister_server {
        DeeperNode::<T>::setup_region_map();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        DeeperNode::<T>::register_device(RawOrigin::Signed(user.clone()).into(), vec![1, 2, 3, 4], "US".as_bytes().to_vec())?;
        let node = DeeperNode::<T>::device_info(user.clone());
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());
        DeeperNode::<T>::register_server(RawOrigin::Signed(user.clone()).into(), 1)?;
    }: _(RawOrigin::Signed(user.clone()))
    verify {
    }

    im_online {
        let user = create_funded_user::<T>("user",USER_SEED, 100);
    }:_(RawOrigin::Signed(user.clone()))
    verify {

    }

    report_credit_proof {
        let admin = <T as pallet_credit_accumulation::Config>::AccountCreator::create_account("Alice");
        let bob = <T as pallet_credit_accumulation::Config>::AccountCreator::create_account("Bob");
        assert_ok!(pallet_credit_accumulation::Pallet::<T>::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob,
        ));
        let signature: [u8; 64] = hex_literal::hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");

    }: _(RawOrigin::Signed(admin.clone()),0,
    signature.into(), 1655007560, 1073741824000000, 4294967295)
    verify {
        assert_eq!(DeviceCreditProof::<T>::get(&admin), (1655007560, 1073741824000000, 4294967295));
    }

    reward_mapping {
        let admin = <T as pallet_credit_accumulation::Config>::AccountCreator::create_account("Alice");
        let bob = <T as pallet_credit_accumulation::Config>::AccountCreator::create_account("Bob");
        assert_ok!(pallet_credit_accumulation::Pallet::<T>::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob,
        ));
        let signature: [u8; 64] = hex_literal::hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");

        let evm_address = H160::from_str("1000000000000000000000000000000000000001").unwrap();
    }: reward_mapping(RawOrigin::Signed(admin.clone()), 0, signature.into(), evm_address)
    verify {
        assert_eq!(RewardsAccountsDeepertoEVM::<T>::get(&admin), Some(evm_address));
    }

    get_npow_reward {
        let admin = <T as pallet_credit_accumulation::Config>::AccountCreator::create_account("Alice");
        let bob = <T as pallet_credit_accumulation::Config>::AccountCreator::create_account("Bob");
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&admin, existential_deposit*2u32.into());

        assert_ok!(pallet_credit_accumulation::Pallet::<T>::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob,
        ));
        let signature: [u8; 64] = hex_literal::hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");
        let evm_address = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let _ = DeeperNode::<T>::reward_mapping(RawOrigin::Signed(admin.clone()).into(), 0, signature.into(), evm_address);
    }: _(RawOrigin::Signed(admin))
    verify {
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
            assert_ok!(Pallet::<Test>::test_benchmark_register_device());
            assert_ok!(Pallet::<Test>::test_benchmark_unregister_device());
            assert_ok!(Pallet::<Test>::test_benchmark_register_server());
            assert_ok!(Pallet::<Test>::test_benchmark_update_server());
            assert_ok!(Pallet::<Test>::test_benchmark_unregister_server());
            assert_ok!(Pallet::<Test>::test_benchmark_im_online());
        });
    }
}
