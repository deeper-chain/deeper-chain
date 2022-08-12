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

use crate::Pallet as UserPriv;

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::EnsureOrigin;
use frame_system::RawOrigin;
use node_primitives::user_privileges::{Privilege, PrivilegeMapping, UserPrivilegeInterface};
use sp_runtime::traits::StaticLookup;

benchmarks! {
    set_user_privilege {
        let user: T::AccountId = account("user", 0, 2);
        let user_lookup = T::Lookup::unlookup(user.clone());
        let origin = T::ForceOrigin::successful_origin();
    }: _<T::Origin>(origin, user_lookup, PrivilegeMapping::LockerMember)
    verify {
        assert_eq!(UserPriv::<T>::has_privilege(&user, Privilege::LockerMember),true);
    }

    clear_user_privilege {
        let user: T::AccountId = account("user", 0, 2);
        let user_lookup = T::Lookup::unlookup(user.clone());
        let origin = T::ForceOrigin::successful_origin();
    }: _<T::Origin>(origin, user_lookup)
    verify {
        assert_eq!(UserPriv::<T>::has_privilege(&user, Privilege::LockerMember),false);
    }

    set_evm_privilege {
        let user: T::AccountId = account("user", 0, 1);
        let user_lookup = T::Lookup::unlookup(user.clone());
        let origin = T::ForceOrigin::successful_origin();
        let _ = UserPriv::<T>::set_user_privilege(origin, user_lookup, PrivilegeMapping::EvmAddressSetter);
    }: _(RawOrigin::Signed(user), H160::from_low_u64_be(88), PrivilegeMapping::LockerMember)
    verify {
        assert_eq!(UserPriv::<T>::has_evm_privilege(&H160::from_low_u64_be(88), Privilege::LockerMember),true);
    }

    clear_evm_privilege {
        let user: T::AccountId = account("user", 0, 1);
        let user_lookup = T::Lookup::unlookup(user.clone());
        let origin = T::ForceOrigin::successful_origin();
        let _ = UserPriv::<T>::set_user_privilege(origin, user_lookup, PrivilegeMapping::EvmAddressSetter);
    }: _(RawOrigin::Signed(user), H160::from_low_u64_be(88))
    verify {
        assert_eq!(UserPriv::<T>::has_evm_privilege(&H160::from_low_u64_be(88), Privilege::LockerMember),false);
    }

    impl_benchmark_test_suite!(UserPriv, crate::tests::new_test_ext(), crate::tests::Test);
}
