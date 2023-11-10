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
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_runtime::traits::StaticLookup;
use sp_runtime::traits::UniqueSaturatedInto;

use crate::Pallet as Adsc;
use frame_support::traits::{
    fungibles::{Create, Mutate},
    Currency,
};
use node_primitives::{credit::H160, user_privileges::Privilege};

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;

pub fn create_funded_user<T: Config>(
    string: &'static str,
    n: u32,
    _balance_factor: u32,
) -> T::AccountId {
    let user = account(string, n, SEED);
    // let balance = <T as pallet::Config>::Currency::minimum_balance() * balance_factor.into();
    // <T as pallet::Config>::Currency::make_free_balance_be(&user, balance);
    // <T as pallet::Config>::Currency::issue(balance);
    user
}

benchmarks! {
    where_clause {where T: pallet_user_privileges::Config}

    add_adsc_staking_account  {
        let admin = create_funded_user::<T>("admin",USER_SEED, 11);
        let admin_lookup = T::Lookup::unlookup(admin.clone());
        let user = create_funded_user::<T>("user",USER_SEED, 10);
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::CreditAdmin);
    }: _(RawOrigin::Signed(admin.clone()), user.clone())
    verify {
        assert!(AdscStakers::<T>::contains_key(user))
    }

    set_reward_period {
        let admin = create_funded_user::<T>("admin",USER_SEED, 11);
        let admin_lookup = T::Lookup::unlookup(admin.clone());
        let user = create_funded_user::<T>("user",USER_SEED, 10);
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::CreditAdmin);
    }: _(RawOrigin::Signed(admin.clone()), 100)
    verify {
        assert_eq!(CurrentRewardPeriod::<T>::get(),100)
    }

    set_half_reward_target {
        let admin = create_funded_user::<T>("admin",USER_SEED, 11);
        let admin_lookup = T::Lookup::unlookup(admin.clone());
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::CreditAdmin);
    }: _(RawOrigin::Signed(admin.clone()), 100u32.into())
    verify {
        assert_eq!(CurrentHalfTarget::<T>::get(),100u32.into())
    }

    set_base_reward {
        let admin = create_funded_user::<T>("admin",USER_SEED, 11);
        let admin_lookup = T::Lookup::unlookup(admin.clone());
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::CreditAdmin);
    }: _(RawOrigin::Signed(admin.clone()), 100u32.into())
    verify {
        assert_eq!(CurrentAdscBaseReward::<T>::get(),100u32.into())
    }

    set_exchange_rate {
        let admin = create_funded_user::<T>("admin",USER_SEED, 11);
        let admin_lookup = T::Lookup::unlookup(admin.clone());
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::CreditAdmin);
    }: _(RawOrigin::Signed(admin.clone()), (2,1))
    verify {

    }

    bridge_burn_adsc {
        let user = create_funded_user::<T>("user",USER_SEED, 10);
        let admin = create_funded_user::<T>("admin",USER_SEED, 11);
        let admin_lookup = T::Lookup::unlookup(admin.clone());
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),admin_lookup,Privilege::CreditAdmin);

        let _ = T::AdscCurrency::create(
            T::AdscId::get(),
            admin.clone(),
            true,
            1_000u32.into(),
        );
        T::AdscCurrency::mint_into(T::AdscId::get(), &user, 20000u32.into())?;

    }: _(RawOrigin::Signed(admin.clone()),user.clone(),H160::default(), 10000u32.into())
    verify {

    }

    swap_adsc_to_dpr {
        let user = create_funded_user::<T>("user",USER_SEED, 10);
        let _ = T::AdscCurrency::create(
            T::AdscId::get(),
            user.clone(),
            true,
            1_000u32.into(),
        );
        let min_balance = <<T as pallet::Config>::DprCurrency as Currency<T::AccountId>>::minimum_balance();
        let amount: u128 = min_balance.unique_saturated_into();
        let _ = T::AdscCurrency::mint_into(T::AdscId::get(), &user, (amount*3).unique_saturated_into());
        let _ = T::DprCurrency::deposit_creating(&Adsc::<T>::account_id(),min_balance*10u32.into());
    }: _(RawOrigin::Signed(user.clone()), (amount*2).unique_saturated_into())
    verify {
    }
}
