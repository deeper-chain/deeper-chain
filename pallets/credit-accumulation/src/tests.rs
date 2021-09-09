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

use crate::{mock::*, testing_utils::*, Error};
use frame_support::{
    assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, error::BadOrigin,
};
use frame_system::RawOrigin;
use hex_literal::hex;

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
