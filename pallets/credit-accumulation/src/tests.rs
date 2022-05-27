// Copyright (C) 2021 Deeper Network Inc.
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

use crate::{mock::*, testing_utils::*, AtmosNonce, Error};
use frame_support::codec::Encode;
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, error::BadOrigin,
};
use frame_system::RawOrigin;
use hex_literal::hex;
use pallet_credit::LastCreditUpdateTimestamp;
use pallet_deeper_node::OnboardTime;
use sp_core::testing::SR25519;
use sp_io::crypto::sr25519_sign;

#[test]
fn add_credit_by_traffic() {
    new_test_ext().execute_with(|| {
        // OK
        assert_ok!(CreditAccumulation::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob(),
        ));

        // InvalidAtomosNonce
        let nonce: u64 = 1;
        let signature: [u8; 64] = hex!("2623f985877cc214821b3c01e364ae14a88a2042222a4bcf7580b48e407c2764c8527be4157c4953378a43f2f7da896d7c0cd268e69f70e601b44db020a05889");
        assert_eq!(CreditAccumulation::add_credit_by_traffic(Origin::signed(alice()), nonce, signature.into()),
        Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidAtomosNonce)));

        // OK
        let nonce: u64 = 0;
        let signature: [u8; 64] = hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");
        assert_ok!(CreditAccumulation::add_credit_by_traffic(
            Origin::signed(alice()),
            nonce, signature.into()
        ));

        // InvalidAtomosNonce
        let nonce: u64 = 0;
        let signature: [u8; 64] = hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");
        assert_eq!(CreditAccumulation::add_credit_by_traffic(Origin::signed(alice()), nonce, signature.into()),
        Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidAtomosNonce)));

        // InvalidSignature
        let nonce: u64 = 0;
        let signature: [u8; 64] = hex!("5071a1a526b1d2d1833e4de43d1ce22ad3506de2e10ee4a9c18c0b310c54286b9cb10bfb4ee12be6b93e91337de0fa2ea2edd787d083db36211109bdc8438989");
        assert_eq!(CreditAccumulation::add_credit_by_traffic(Origin::signed(bob()), nonce, signature.into()),
        Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidSignature)));

        // OK
        let nonce: u64 = 1;
        let signature: [u8; 64] = hex!("2623f985877cc214821b3c01e364ae14a88a2042222a4bcf7580b48e407c2764c8527be4157c4953378a43f2f7da896d7c0cd268e69f70e601b44db020a05889");
        assert_ok!(CreditAccumulation::add_credit_by_traffic(
            Origin::signed(alice()),
            nonce, signature.into()
        ));
    });
}

#[test]
fn add_one_credit_one_era() {
    new_test_ext().execute_with(|| {
        assert_ok!(CreditAccumulation::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob(),
        ));

        let nonce: u64 = 1;
        AtmosNonce::<Test>::insert(alice(), nonce);
        OnboardTime::<Test>::insert(alice(), 40); // block2
        LastCreditUpdateTimestamp::<Test>::insert(alice(), 40); // block2

        run_to_block(BLOCKS_PER_ERA+2);
        let signature: [u8; 64] = hex!("34fb245d1d6df01f8177a8e3d29d30a63eb22b9d6c691a97536e1a2805953951194250865971237cf70edb934d76c91744460fe78a7cb86b537ab56146e32b81");
        assert_ok!(CreditAccumulation::add_credit_by_traffic(
            Origin::signed(alice()),
            nonce, signature.into()
        ));

        assert_eq!(400, Credit::last_credit_update_timestamp(alice()).unwrap()); // (72+2)*5+30
        assert_eq!(211, Credit::user_credit(alice()).unwrap().credit);

        let mut events = <frame_system::Pallet<Test>>::events();
        assert_eq!(
            events.pop().expect("should get first events").event,
            crate::tests::Event::from(pallet_credit::Event::CreditDataAddedByTraffic(alice(), 211))
        );

        assert_eq!(
            events.pop().expect("should get second events").event,
            crate::tests::Event::from(pallet_credit::Event::CreditUpdateSuccess(alice(), 211))
        );

        assert_eq!(
            events.pop().expect("should get third events").event,
            crate::tests::Event::from(crate::Event::AtmosSignatureValid(alice()))
        );

        assert!(events.is_empty());
    });
}

#[test]
fn not_add_credit_less_than_one_era() {
    new_test_ext().execute_with(|| {
        assert_ok!(CreditAccumulation::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob(),
        ));

        let nonce: u64 = 1;
        AtmosNonce::<Test>::insert(alice(), nonce);
        OnboardTime::<Test>::insert(alice(), 40); // block2
        LastCreditUpdateTimestamp::<Test>::insert(alice(), 40); // block2

        run_to_block(BLOCKS_PER_ERA+1);
        let signature: [u8; 64] = hex!("34fb245d1d6df01f8177a8e3d29d30a63eb22b9d6c691a97536e1a2805953951194250865971237cf70edb934d76c91744460fe78a7cb86b537ab56146e32b81");
        assert_ok!(CreditAccumulation::add_credit_by_traffic(
            Origin::signed(alice()),
            nonce, signature.into()
        ));

        assert_eq!(40, Credit::last_credit_update_timestamp(alice()).unwrap()); // (72+2)*5+30
        assert_eq!(210, Credit::user_credit(alice()).unwrap().credit);

        let mut events = <frame_system::Pallet<Test>>::events();
        assert_eq!(
            events.pop().expect("should get first events").event,
            crate::tests::Event::from(crate::Event::AtmosSignatureValid(alice()))
        );

        assert!(events.is_empty());
    });
}

#[test]
fn only_add_one_credit_even_if_more_eras() {
    new_test_ext().execute_with(|| {
        assert_ok!(CreditAccumulation::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob(),
        ));

        let nonce: u64 = 1;
        AtmosNonce::<Test>::insert(alice(), nonce);
        OnboardTime::<Test>::insert(alice(), 40); // block2
        LastCreditUpdateTimestamp::<Test>::insert(alice(), 40); // block2

        run_to_block(2*BLOCKS_PER_ERA+3);
        let signature: [u8; 64] = hex!("34fb245d1d6df01f8177a8e3d29d30a63eb22b9d6c691a97536e1a2805953951194250865971237cf70edb934d76c91744460fe78a7cb86b537ab56146e32b81");
        assert_ok!(CreditAccumulation::add_credit_by_traffic(
            Origin::signed(alice()),
            nonce, signature.into()
        ));

        assert_eq!(765, Credit::last_credit_update_timestamp(alice()).unwrap()); // (2*72+3)*5+30
        assert_eq!(211, Credit::user_credit(alice()).unwrap().credit);

        let mut events = <frame_system::Pallet<Test>>::events();
        assert_eq!(
            events.pop().expect("should get first events").event,
            crate::tests::Event::from(pallet_credit::Event::CreditDataAddedByTraffic(alice(), 211))
        );

        assert_eq!(
            events.pop().expect("should get second events").event,
            crate::tests::Event::from(pallet_credit::Event::CreditUpdateSuccess(alice(), 211))
        );

        assert_eq!(
            events.pop().expect("should get third events").event,
            crate::tests::Event::from(crate::Event::AtmosSignatureValid(alice()))
        );

        assert!(events.is_empty());
    });
}

#[test]
fn set_atmos_pubkey() {
    new_test_ext().execute_with(|| {
        // OK
        assert_ok!(CreditAccumulation::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob(),
        ));

        // BadOrigin
        assert_noop!(
            CreditAccumulation::set_atmos_pubkey(Origin::signed(alice()), bob(),),
            BadOrigin
        );
    });
}

#[test]
fn verify_atomos_signature() {
    new_test_ext().execute_with(|| {
        assert_ok!(CreditAccumulation::set_atmos_pubkey(
            RawOrigin::Root.into(),
            bob(),
        ));
        let nonce: u64 = 1;
        let atomos_accountid = CreditAccumulation::atmos_accountid().unwrap();
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&atomos_accountid.encode());
        let pub_key = sp_core::sr25519::Public::from_raw(pk);

        let mut data = Vec::new();
        data.extend_from_slice(&bob().encode());
        data.extend_from_slice(&nonce.to_be_bytes());
        data.extend_from_slice(&alice().encode());
        let msg = sp_io::hashing::blake2_256(&data);

        let sig = sr25519_sign(SR25519, &pub_key, &msg).unwrap();
        assert_ok!(CreditAccumulation::verify_atomos_signature(
            nonce,
            &sig.encode(),
            alice()
        ));
    });
}
