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
use crate::Pallet as Bridge;
use codec::Encode;
pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use hex_literal::hex;
pub use node_primitives::{AccountId, Signature};
use sp_core::{sr25519, Hasher, H160};
use sp_io::crypto::{sr25519_generate, sr25519_sign};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{MultiSignature, MultiSigner};
use sp_std::{convert::TryFrom, vec::Vec};
use types::Status;
type AccountPublic = <Signature as Verify>::Signer;

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;
const ETH_ADDRESS: &[u8; 20] = b"0x00b46c2526ebb8f4c9";
const ETH_MESSAGE_ID: &[u8; 32] = b"0x5617efe391571b5dc8230db92ba65b";
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
    //use sp_core::sr25519::Public;
    // return
    sr25519_generate(0.into(), Some(seed)).into()
}

pub fn create_sr25519_signature(payload: &[u8], pubkey: MultiSigner) -> MultiSignature {
    let srpubkey = sr25519::Public::try_from(pubkey).unwrap();
    let srsig = sr25519_sign(0.into(), &srpubkey, payload).unwrap();
    srsig.into()
}

benchmarks! {
    set_transfer {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = T::Currency::minimum_balance() * 49u32.into();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        let transfer_hash = (&user, &eth_address, amount, <pallet_timestamp::Module<T>>::get())
        .using_encoded(<T as frame_system::Config>::Hashing::hash);
        whitelist_account!(user);
    }: _(RawOrigin::Signed(user.clone()), eth_address, amount)
    verify {
        assert!(TransferMessages::<T>::contains_key(transfer_hash));
    }

    multi_signed_mint {
        let message_id = ETH_MESSAGE_ID.using_encoded(<T as frame_system::Config>::Hashing::hash);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = T::Currency::minimum_balance() * 99u32.into();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        let validator1 = create_funded_user::<T>("user",USER_SEED, 100);
    }:  _(RawOrigin::Signed(validator1), message_id, eth_address, user,amount)
    verify {
        let message = Bridge::<T>::messages(message_id);
        assert_eq!(message.status, Status::Pending);
    }

    update_limits {
        let max_tx_value = 10u32;
        let day_max_limit = 20u32;
        let day_max_limit_for_one_address = 5u32;
        let max_pending_tx_limit = 40u32;
        let min_tx_value = 1u32;
        let validator1 = create_funded_user::<T>("user",USER_SEED, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+1, 100);
        // TODO Create Account by using private of validator in bridge
        //let pair = sr25519::Pair::from_seed(&hex!("73e79288db1c1b7d0ed3cb38d149f1de6c0a771406a3fee330c38b4e37643a9a"));
        //let account_id = AccountPublic::from(pair.public()).into_account();
        Bridge::<T>::update_limits(
            RawOrigin::Signed(validator1).into(),
            max_tx_value.into(),
            day_max_limit.into(), day_max_limit_for_one_address.into(), max_pending_tx_limit.into(), min_tx_value.into())?;
    }:  _(RawOrigin::Signed(validator2), max_tx_value.into(), day_max_limit.into(), day_max_limit_for_one_address.into(), max_pending_tx_limit.into(), min_tx_value.into())
    verify {
        assert_eq!(Bridge::<T>::current_limits().max_tx_value, 10u32.into());
    }

    approve_transfer {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = T::Currency::minimum_balance() * 49u32.into();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        let validator1 = create_funded_user::<T>("user",USER_SEED, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+1, 100);

        Bridge::<T>::set_transfer(RawOrigin::Signed(user).into(), eth_address, amount)?;
        let sub_message_id = Bridge::<T>::message_id_by_transfer_id(0);
        Bridge::<T>::approve_transfer(RawOrigin::Signed(validator1).into(), sub_message_id)?;
    }: _(RawOrigin::Signed(validator2), sub_message_id)
    verify {
        let message = Bridge::<T>::messages(sub_message_id);
        assert_eq!(message.status, Status::Approved);
    }

    update_validator_list {
        let eth_message_id = ETH_MESSAGE_ID.using_encoded(<T as frame_system::Config>::Hashing::hash);
        const QUORUM: u64 = 3;
        let validator1 = create_funded_user::<T>("user",USER_SEED, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+1, 100);
        let user1 = create_funded_user::<T>("user",USER_SEED+2, 100);
        let user2 = create_funded_user::<T>("user",USER_SEED+3, 100);
        let user3 = create_funded_user::<T>("user",USER_SEED+4, 100);
        let user4 = create_funded_user::<T>("user",USER_SEED+5, 100);

        Bridge::<T>::update_validator_list(
            RawOrigin::Signed(validator1).into(),
            eth_message_id, QUORUM,
            vec![user1.clone(), user2.clone(), user3.clone(), user4.clone()])?;
        let id = Bridge::<T>::message_id_by_transfer_id(0);
        let message = Bridge::<T>::validator_history(id);
        assert_eq!(message.status, Status::Pending);
    }: _(RawOrigin::Signed(validator2), eth_message_id, QUORUM, vec![user1, user2, user3, user4])
    verify {
        let message = Bridge::<T>::validator_history(id);
        assert_eq!(message.status, Status::Confirmed);
        assert_eq!(Bridge::<T>::validators_count(), 4);
    }

    pause_bridge {
        let validator1 = create_funded_user::<T>("user",USER_SEED, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+1, 100);

        Bridge::<T>::pause_bridge(RawOrigin::Signed(validator1).into())?;
        assert_eq!(Bridge::<T>::bridge_transfers_count(), 1);
        assert_eq!(Bridge::<T>::bridge_is_operational(), true);
        let id = Bridge::<T>::message_id_by_transfer_id(0);
        let message = Bridge::<T>::bridge_messages(id);
        assert_eq!(message.status, Status::Pending);
    }: _(RawOrigin::Signed(validator2))
    verify {
        assert_eq!(Bridge::<T>::bridge_is_operational(), false);
        let message = Bridge::<T>::bridge_messages(id);
        assert_eq!(message.status, Status::Confirmed);
    }

    resume_bridge {
        let validator1 = create_funded_user::<T>("user",USER_SEED, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+1, 100);

        assert_eq!(Bridge::<T>::bridge_is_operational(), true);
        Bridge::<T>::pause_bridge(RawOrigin::Signed(validator1.clone()).into())?;
        Bridge::<T>::pause_bridge(RawOrigin::Signed(validator2.clone()).into())?;
        assert_eq!(Bridge::<T>::bridge_is_operational(), false);

        Bridge::<T>::resume_bridge(RawOrigin::Signed(validator1).into())?;
    }: _(RawOrigin::Signed(validator2))
    verify {
        assert_eq!(Bridge::<T>::bridge_is_operational(), true);
    }

    confirm_transfer {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = T::Currency::minimum_balance() * 49u32.into();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        let validator1 = create_funded_user::<T>("user",USER_SEED+1, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+2, 100);

        Bridge::<T>::set_transfer(RawOrigin::Signed(user.clone()).into(), eth_address, amount)?;
        let sub_message_id = Bridge::<T>::message_id_by_transfer_id(0);
        Bridge::<T>::approve_transfer(RawOrigin::Signed(validator1.clone()).into(), sub_message_id)?;
        Bridge::<T>::approve_transfer(RawOrigin::Signed(validator2.clone()).into(), sub_message_id)?;
        let mut message = Bridge::<T>::messages(sub_message_id);
        assert_eq!(message.status, Status::Approved);

        // TODO Delete validator3,validator4
        // Current validator1, validator2 is not in ValidaotrsData
        let validator3 = create_funded_user::<T>("user",USER_SEED+3, 100);
        let validator4 = create_funded_user::<T>("user",USER_SEED+4, 100);
        Bridge::<T>::confirm_transfer(RawOrigin::Signed(validator3.clone()).into(), sub_message_id)?;

        message = Bridge::<T>::messages(sub_message_id);
        assert_eq!(message.status, Status::Confirmed);
        let transfer = Bridge::<T>::transfers(1);
        assert_eq!(transfer.open, true);
    }: _(RawOrigin::Signed(validator4), sub_message_id)
    verify {
        assert_eq!(T::Currency::free_balance(&user), T::Currency::minimum_balance() * 51u32.into());
    }

    cancel_transfer {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = T::Currency::minimum_balance() * 49u32.into();
        let user = create_funded_user::<T>("user",USER_SEED, 100);
        let validator1 = create_funded_user::<T>("user",USER_SEED+1, 100);
        let validator2 = create_funded_user::<T>("user",USER_SEED+2, 100);

        Bridge::<T>::set_transfer(RawOrigin::Signed(user.clone()).into(), eth_address, amount)?;
        let sub_message_id = Bridge::<T>::message_id_by_transfer_id(0);
        Bridge::<T>::approve_transfer(RawOrigin::Signed(validator1.clone()).into(), sub_message_id)?;
        Bridge::<T>::approve_transfer(RawOrigin::Signed(validator2.clone()).into(), sub_message_id)?;
        let message = Bridge::<T>::messages(sub_message_id);
        assert_eq!(message.status, Status::Approved);

        // TODO Delete validator3,validator4
        // Current validator1, validator2 is not in ValidaotrsData
        let validator3 = create_funded_user::<T>("user",USER_SEED+3, 100);
        let validator4 = create_funded_user::<T>("user",USER_SEED+4, 100);
        Bridge::<T>::cancel_transfer(RawOrigin::Signed(validator3.clone()).into(), sub_message_id)?;
    }: _(RawOrigin::Signed(validator4), sub_message_id)
    verify {
        let message = Bridge::<T>::messages(sub_message_id);
        assert_eq!(message.status, Status::Canceled);
    }
}
