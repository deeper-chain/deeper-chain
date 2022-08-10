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
use frame_support::traits::{Currency, Get, LockableCurrency, WithdrawReasons};
use frame_system::RawOrigin;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::StaticLookup;

use node_primitives::{credit::CreditInterface, user_privileges::Privilege};
use pallet_user_privileges::Pallet as UserPrivileges;
use scale_info::prelude::string::ToString;

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;
// existential deposit multiplier
const ED_MULTIPLIER: u32 = 10;

const FORCE_LOCK_ID: [u8; 8] = *b"abcdefgh";

benchmarks! {
    where_clause { where T: Config, T: pallet_balances::Config + pallet_user_privileges::Config+ pallet_evm::Config }
    force_reserve_by_member {
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let unlocker: T::AccountId = account("unlocker", 1, SEED);
        let unlocker_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(unlocker.clone());
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&unlocker, existential_deposit);
        let _ = UserPrivileges::<T>::set_user_privilege(RawOrigin::Root.into(),unlocker_lookup,Privilege::LockerMember);
        let source: T::AccountId = account("locked", 0, SEED);
        let source_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(source.clone());
        // Give some multiple of the existential deposit + creation fee + transfer fee
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&source, balance);
    }: force_reserve_by_member(RawOrigin::Signed(unlocker), source_lookup, balance)
    verify {
        assert_eq!(pallet_balances::Account::<T>::get(&source).free, T::Balance::zero());
    }

    force_remove_lock {
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let source: T::AccountId = account("locked", 0, SEED);
        let source_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(source.clone());
        // Give some multiple of the existential deposit + creation fee + transfer fee
        let balance = existential_deposit.saturating_mul(ED_MULTIPLIER.into());
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&source, balance);
        let _ = <T as pallet::Config>::Currency::set_lock(FORCE_LOCK_ID,&source, existential_deposit,WithdrawReasons::all());
    }: force_remove_lock(RawOrigin::Root, FORCE_LOCK_ID,source_lookup)
    verify {
        assert_eq!(<T as pallet::Config>::Currency::free_balance(&source), balance);
    }

    set_release_limit_parameter {
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let single_limit = existential_deposit * 10u32.into();
        let daily_limit = existential_deposit * 1000u32.into();
    }: set_release_limit_parameter(RawOrigin::Root, single_limit, daily_limit)
    verify {
        assert_eq!(SingleMaxLimit::<T>::get(),single_limit);
        assert_eq!(DailyMaxLimit::<T>::get(),daily_limit);
    }

    unstaking_release {
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let admin: T::AccountId = account("a", 0, SEED);
        let admin_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(admin.clone());

        let _ = UserPrivileges::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::ReleaseSetter);
        let single_limit = existential_deposit * 10u32.into();
        let daily_limit = existential_deposit * 1000u32.into();
        <SingleMaxLimit<T>>::put(single_limit);
        <DailyMaxLimit<T>>::put(daily_limit);
        let checked_account: T::AccountId = account("a", 100, USER_SEED);
        T::CreditInterface::add_or_update_credit(checked_account.clone(),99);
        let rinfo= ReleaseInfo::<T>::new(checked_account.clone(),2,0,existential_deposit * 10u32.into());
    }: unstaking_release(RawOrigin::Signed(admin), rinfo)
    verify {
        assert_eq!(AccountsReleaseInfo::<T>::contains_key(&checked_account),true);
    }

    burn_for_ezc {
        let existential_deposit = T::MinimumBurnedDPR::get();
        let user: T::AccountId = account("user", 0, SEED);
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&user, existential_deposit*2u32.into());
    }: burn_for_ezc(RawOrigin::Signed(user.clone()), existential_deposit*1u32.into(), H160::zero())
    verify {
        assert_eq!(<T as pallet::Config>::Currency::free_balance(&user),existential_deposit);
    }

    npow_mint {
        let account: T::AccountId = account("b", 1, USER_SEED);
        let account_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(account.clone());

        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let _ = UserPrivileges::<T>::set_user_privilege(RawOrigin::Root.into(),account_lookup,Privilege::NpowMint);
        let dpr = existential_deposit * 10u32.into();
    }: npow_mint(RawOrigin::Signed(account.clone()), account.clone(), dpr)
    verify {
    }

    bridge_deeper_to_other {
        let user1: T::AccountId = account("b", 1, USER_SEED);
        let user2: T::AccountId = account("b", 2, USER_SEED);
        let account_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user1.clone());
        let _ = UserPrivileges::<T>::set_user_privilege(RawOrigin::Root.into(),account_lookup,Privilege::BridgeAdmin);
        BridgeFundAddreess::<T>::put(user1.clone());
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&user1, existential_deposit*2u32.into());
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&user2, existential_deposit*2u32.into());

    }: bridge_deeper_to_other(RawOrigin::Signed(user1.clone()), H160::zero(),user2,existential_deposit,"test".to_string())
    verify {
    }

    bridge_other_to_deeper {
        let user1: T::AccountId = account("b", 1, USER_SEED);
        let user2: T::AccountId = account("b", 2, USER_SEED);
        let account_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user1.clone());
        let _ = UserPrivileges::<T>::set_user_privilege(RawOrigin::Root.into(),account_lookup,Privilege::BridgeAdmin);
        BridgeFundAddreess::<T>::put(user1.clone());
        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&user1, existential_deposit*2u32.into());
        let _ = <T as pallet::Config>::Currency::make_free_balance_be(&user2, existential_deposit*2u32.into());

    }: bridge_other_to_deeper(RawOrigin::Signed(user1.clone()), user2,H160::zero(),existential_deposit,"test".to_string())
    verify {
    }

    set_dpr_price {
        let user: T::AccountId = account("b", 1, USER_SEED);
        let account_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());
        let _ = UserPrivileges::<T>::set_user_privilege(RawOrigin::Root.into(),account_lookup,Privilege::CreditAdmin);

        let existential_deposit = <T as pallet::Config>::Currency::minimum_balance();
    }: _(RawOrigin::Signed(user), existential_deposit, H160::zero() )
    verify {
        assert_eq!(DprPrice::<T>::get(),Some(existential_deposit));
    }

    impl_benchmark_test_suite!(Operation, crate::tests::new_test_ext(), crate::tests::Test);
}
