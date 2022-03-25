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

//! Balances pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency, WithdrawReasons};
use frame_system::RawOrigin;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::StaticLookup;

use crate::Pallet as Op;

const SEED: u32 = 0;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;
// lockid
const FORCE_LOCK_ID: [u8; 8] = *b"forcelck";

benchmarks! {
    where_clause { where T: Config, T: pallet_balances::Config }
    force_lock {
        let existential_deposit = T::Currency::minimum_balance();
        let unlocker: T::AccountId = account("unlocker", 1, SEED);
        let _ = T::Currency::make_free_balance_be(&unlocker, existential_deposit);
        let  _ = Op::<T>::set_lock_members(RawOrigin::Root.into(),vec!(unlocker.clone()));
        let source: T::AccountId = account("locked", 0, SEED);
        let source_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(source.clone());
        // Give some multiple of the existential deposit + creation fee + transfer fee
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = T::Currency::make_free_balance_be(&source, balance);
    }: force_lock(RawOrigin::Signed(unlocker), source_lookup, balance)
    verify {
        assert_eq!(pallet_balances::Account::<T>::get(&source).fee_frozen, pallet_balances::Account::<T>::get(&source).free);
    }

    force_remove_lock {
        let existential_deposit = T::Currency::minimum_balance();
        let source: T::AccountId = account("locked", 0, SEED);
        let source_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(source.clone());
        // Give some multiple of the existential deposit + creation fee + transfer fee
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = T::Currency::make_free_balance_be(&source, balance);
        let _ = T::Currency::set_lock(FORCE_LOCK_ID,&source, existential_deposit,WithdrawReasons::all());
    }: force_remove_lock(RawOrigin::Root, FORCE_LOCK_ID,source_lookup)
    verify {
        assert_eq!(T::Currency::free_balance(&source), balance);
    }

    force_unreserve {
        let existential_deposit = T::Currency::minimum_balance();
        let source: T::AccountId = account("reserved", 0, SEED);
        let source_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(source.clone());
        // Give some multiple of the existential deposit + creation fee + transfer fee
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = T::Currency::make_free_balance_be(&source, balance);
        let _ = T::Currency::reserve(&source, existential_deposit);
    }: force_unreserve(RawOrigin::Root, source_lookup, T::Currency::minimum_balance())
    verify {
        assert_eq!(T::Currency::free_balance(&source), balance);
    }

    set_lock_members {
        let existential_deposit = T::Currency::minimum_balance();
        let user_a: T::AccountId = account("a", 0, SEED);
        let user_b: T::AccountId = account("b", 0, SEED);
        // let a_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user_a.clone());
        // let b_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user_b.clone());
        let _ = T::Currency::make_free_balance_be(&user_a, existential_deposit);
        let _ = T::Currency::make_free_balance_be(&user_b, existential_deposit);
    }: set_lock_members(RawOrigin::Root, vec!(user_a,user_b))
    verify {
        assert_eq!(Op::<T>::lock_member_whitelist().len(), 2);
    }

    impl_benchmark_test_suite!(Op, crate::tests::new_test_ext(), crate::tests::Test);
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::mock::{new_test_ext, Test};
//     use frame_support::assert_ok;

//     #[test]
//     fn test_benchmarks() {
//         new_test_ext().execute_with(|| {
//             assert_ok!(Pallet::<Test>::test_benchmark_update_credit_setting());
//             assert_ok!(Pallet::<Test>::test_benchmark_update_credit_setting());
//         });
//     }
// }
