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

//! Micropayment pallet benchmarking.

use super::*;
use crate::Module as CreditAccumulation;
pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use hex_literal::hex;
use pallet_micropayment::AccountCreator;
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
    add_credit_by_traffic {
        let alice = create_funded_user::<T>("Alice", 100);
        let bob = create_funded_user::<T>("Bob", 100);
        // OK
        assert_ok!(CreditAccumulation::<T>::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob,
        ));

        // OK
        let nonce: u64 = 0;
        let signature: [u8; 64] = hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");
    }: _(RawOrigin::Signed(alice.clone()), nonce, signature.into())
    verify {
        assert_eq!(
            CreditAccumulation::<T>::atmos_nonce(alice), Some(1)
        );
    }

    set_atmos_pubkey {
        let bob = create_funded_user::<T>("Bob", 100);
    }: _(RawOrigin::Root, bob.clone())
    verify {
        assert_eq!(
            CreditAccumulation::<T>::atmos_accountid(), Some(bob)
        );
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
            assert_ok!(Pallet::<Test>::test_benchmark_add_credit_by_traffic());
            assert_ok!(Pallet::<Test>::test_benchmark_set_atmos_pubkey());
        });
    }
}
