// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

//! Macro for creating the tests for the module.
#[cfg(test)]
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use frame_support::assert_ok;
use frame_support::traits::ConstU32;
use frame_support::{parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use node_primitives::user_privileges::{Privilege, UserPrivilegeInterface};

use super::*;
use crate::{self as pallet_user_privileges};
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        UserPrivileges: pallet_user_privileges::{Pallet, Call, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_user_privileges::Config for Test {
    type Event = Event;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    t.into()
}

#[test]
fn op_user_privilege() {
    new_test_ext().execute_with(|| {
        assert_ok!(UserPrivileges::set_user_privilege(
            Origin::root(),
            1,
            Privilege::LockerMember
        ));
        assert_ok!(UserPrivileges::set_user_privilege(
            Origin::root(),
            1,
            Privilege::ReleaseSetter
        ));
        assert_eq!(
            UserPrivileges::has_privilege(&1, Privilege::ReleaseSetter),
            true
        );
        assert_eq!(
            UserPrivileges::has_privilege(&1, Privilege::LockerMember),
            true
        );
        assert_ok!(UserPrivileges::unset_user_privilege(
            Origin::root(),
            1,
            Privilege::ReleaseSetter
        ));
        assert_eq!(
            UserPrivileges::has_privilege(&1, Privilege::ReleaseSetter),
            false
        );
        assert_ok!(UserPrivileges::clear_user_privilege(Origin::root(), 1));
        assert_eq!(
            UserPrivileges::has_privilege(&1, Privilege::LockerMember),
            false
        );

        assert_ok!(UserPrivileges::set_user_privilege(
            Origin::root(),
            1,
            Privilege::EvmAddressSetter
        ));

        assert_ok!(UserPrivileges::set_evm_privilege(
            Origin::signed(1),
            H160::from_low_u64_be(88),
            Privilege::EvmCreditOperation
        ));
        assert_ok!(UserPrivileges::set_evm_privilege(
            Origin::signed(1),
            H160::from_low_u64_be(88),
            Privilege::ReleaseSetter
        ));
        assert_eq!(
            UserPrivileges::has_evm_privilege(
                &H160::from_low_u64_be(88),
                Privilege::EvmCreditOperation
            ),
            true
        );
        assert_eq!(
            UserPrivileges::has_evm_privilege(&H160::from_low_u64_be(88), Privilege::ReleaseSetter),
            true
        );

        assert_ok!(UserPrivileges::unset_evm_privilege(
            Origin::signed(1),
            H160::from_low_u64_be(88),
            Privilege::EvmCreditOperation
        ));
        assert_eq!(
            UserPrivileges::has_evm_privilege(
                &H160::from_low_u64_be(88),
                Privilege::EvmCreditOperation
            ),
            false
        );
        assert_ok!(UserPrivileges::clear_evm_privilege(
            Origin::signed(1),
            H160::from_low_u64_be(88)
        ));
        assert_eq!(
            UserPrivileges::has_evm_privilege(&H160::from_low_u64_be(88), Privilege::ReleaseSetter),
            false
        );
    });
}
