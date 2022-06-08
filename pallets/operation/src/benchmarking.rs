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

use frame_benchmarking::{account, benchmarks, Zero};
use frame_support::traits::{Currency, LockableCurrency, WithdrawReasons};
use frame_system::RawOrigin;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::StaticLookup;

use crate::Pallet as Op;

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;

const FORCE_LOCK_ID: [u8; 8] = *b"abcdefgh";

benchmarks! {
    where_clause { where T: Config, T: pallet_balances::Config }
    force_reserve_by_member {
        let existential_deposit = T::Currency::minimum_balance();
        let unlocker: T::AccountId = account("unlocker", 1, SEED);
        let _ = T::Currency::make_free_balance_be(&unlocker, existential_deposit);
        let  _ = Op::<T>::set_reserve_members(RawOrigin::Root.into(),vec!(unlocker.clone()));
        let source: T::AccountId = account("locked", 0, SEED);
        let source_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(source.clone());
        // Give some multiple of the existential deposit + creation fee + transfer fee
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = T::Currency::make_free_balance_be(&source, balance);
    }: force_reserve_by_member(RawOrigin::Signed(unlocker), source_lookup, balance)
    verify {
        assert_eq!(pallet_balances::Account::<T>::get(&source).free, T::Balance::zero());
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

    set_reserve_members {
        let existential_deposit = T::Currency::minimum_balance();
        let user_a: T::AccountId = account("a", 0, SEED);
        let user_b: T::AccountId = account("b", 0, SEED);
        let _ = T::Currency::make_free_balance_be(&user_a, existential_deposit);
        let _ = T::Currency::make_free_balance_be(&user_b, existential_deposit);
    }: set_reserve_members(RawOrigin::Root, vec!(user_a,user_b))
    verify {
        assert_eq!(Op::<T>::lock_member_whitelist().len(), 2);
    }

    set_release_owner_address {
        let user: T::AccountId = account("user", 0, SEED);
    }: set_release_owner_address(RawOrigin::Root, user.clone())
    verify {
        assert_eq!(ReleasePaymentAddress::<T>::get(),Some(user.clone()));
    }

    set_release_limit_parameter {
        let existential_deposit = T::Currency::minimum_balance();
        let single_limit = existential_deposit * 10u32.into();
        let daily_limit = existential_deposit * 1000u32.into();
    }: set_release_limit_parameter(RawOrigin::Root, single_limit, daily_limit)
    verify {
        assert_eq!(SingleMaxLimit::<T>::get(),single_limit);
        assert_eq!(DailyMaxLimit::<T>::get(),daily_limit);
    }

    unstaking_release {
        let existential_deposit = T::Currency::minimum_balance();
        let admin: T::AccountId = account("a", 0, SEED);
        <ReleasePaymentAddress<T>>::put(admin.clone());
        let single_limit = existential_deposit * 10u32.into();
        let daily_limit = existential_deposit * 1000u32.into();
        <SingleMaxLimit<T>>::put(single_limit);
        <DailyMaxLimit<T>>::put(daily_limit);

        let checked_account: T::AccountId = account("a", 100, USER_SEED);
        let rinfo= ReleaseInfo::<T>::new(checked_account.clone(),2,0,existential_deposit * 10u32.into());

    }: _(RawOrigin::Signed(admin), rinfo)
    verify {
        assert_eq!(AccountsReleaseInfo::<T>::contains_key(&checked_account),true);
    }

    burn_for_ezc {
        let existential_deposit = T::Currency::minimum_balance();
        let user: T::AccountId = account("user", 0, SEED);
        let _ = T::Currency::make_free_balance_be(&user, existential_deposit*2u32.into());
    }: burn_for_ezc(RawOrigin::Signed(user.clone()), existential_deposit, H160::zero())
    verify {
        assert_eq!(T::Currency::free_balance(&user),existential_deposit);
    }

    impl_benchmark_test_suite!(Op, crate::tests::new_test_ext(), crate::tests::Test);
}
