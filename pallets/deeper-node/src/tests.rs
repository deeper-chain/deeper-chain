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

use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok, dispatch::DispatchErrorWithPostInfo};
use node_primitives::deeper_node::NodeInterface;
use sp_core::H160;
use std::str::FromStr;

#[test]
fn register_device() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
        // register device
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        let node = DeeperNode::device_info(1);
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());

        // register device with invalid ip (length > 256)
        assert_eq!(
            DeeperNode::register_device(Origin::signed(2), vec![1; 257], "US".as_bytes().to_vec()),
            Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidIP))
        );

        // register device with invalid country code
        assert_eq!(
            DeeperNode::register_device(
                Origin::signed(3),
                vec![1, 2, 3, 4],
                "ZZ".as_bytes().to_vec()
            ),
            Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidCode))
        );

        // register device twice
        assert_ok!(DeeperNode::register_device(
            Origin::signed(4),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::register_device(
            Origin::signed(4),
            vec![1, 3, 4, 5],
            "CA".as_bytes().to_vec()
        ));
        let node = DeeperNode::device_info(4);
        assert_eq!(node.ipv4, vec![1, 3, 4, 5]);
        assert_eq!(node.country, "CA".as_bytes().to_vec());
    });
}

#[test]
fn unregister_device() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
        // unregister a registered device
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::unregister_device(Origin::signed(1)));

        // unregister an unregistered device
        assert_eq!(
            DeeperNode::unregister_device(Origin::signed(2)),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );
    });
}

#[test]
fn register_server() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::register_server(Origin::signed(1), 1));
        let servers = DeeperNode::servers_by_country("US".as_bytes().to_vec());
        let index = servers.iter().position(|x| *x == 1);
        assert_eq!(index, Some(0));

        // register server before register device
        assert_eq!(
            DeeperNode::register_server(Origin::signed(2), 1),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );

        // register server with invalid duration
        assert_ok!(DeeperNode::register_device(
            Origin::signed(3),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_eq!(
            DeeperNode::register_server(Origin::signed(3), 8),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DurationOverflow
            ))
        );
    });
}

#[test]
fn unregister_server() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::register_server(Origin::signed(1), 1));
        assert_ok!(DeeperNode::unregister_server(Origin::signed(1)));

        // register server before register device
        assert_eq!(
            DeeperNode::unregister_server(Origin::signed(2)),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );
    });
}

#[test]
fn update_server() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
        // register device, then update server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::update_server(Origin::signed(1), 1));

        // update server before register device
        assert_eq!(
            DeeperNode::update_server(Origin::signed(2), 1),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );

        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(3),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_eq!(
            DeeperNode::update_server(Origin::signed(3), 10),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DurationOverflow
            ))
        );
    });
}

#[test]
fn im_online() {
    new_test_ext().execute_with(|| {
        assert_eq!(DeeperNode::onboard_time(1), None);
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(DeeperNode::get_im_online(1), Some(0));
        assert_eq!(DeeperNode::onboard_time(1), Some(0));
        assert_eq!(DeeperNode::devices_onboard(), vec![1]);
        run_to_block(1);
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(DeeperNode::get_im_online(1), Some(1));
        assert_eq!(DeeperNode::onboard_time(1), Some(0));
        assert_eq!(DeeperNode::devices_onboard(), vec![1]);
        run_to_block(2);
        assert_ok!(DeeperNode::im_online(Origin::signed(2)));
        assert_eq!(DeeperNode::get_im_online(1), Some(1));
        assert_eq!(DeeperNode::get_im_online(2), Some(2));
        assert_eq!(DeeperNode::onboard_time(2), Some(2));
        assert_eq!(DeeperNode::devices_onboard(), vec![1, 2]);
    });
}

#[test]
fn report_credit_proof() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::report_credit_proof(
            Origin::signed(1),
            0,
            Vec::new(),
            1655007560,
            1073741824000000,
            4294967295
        ));
        assert_eq!(
            DeeperNode::device_credit_proof(1),
            (1655007560, 1073741824000000, 4294967295)
        );
    });
}

#[test]
fn reward_mapping() {
    new_test_ext().execute_with(|| {
        let evm_address = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        assert_ok!(DeeperNode::reward_mapping(
            Origin::signed(1),
            0,
            Vec::new(),
            evm_address
        ));
        assert_eq!(
            DeeperNode::rewards_accounts_deeper_evm(&1),
            Some(evm_address)
        );
    });
}

#[test]
fn reward_mapping_switch_evm_address() {
    new_test_ext().execute_with(|| {
        let evm_old_address = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let evm_new_address = H160::from_str("1000000000000000000000000000000000000002").unwrap();
        assert_ok!(DeeperNode::reward_mapping(
            Origin::signed(1),
            0,
            Vec::new(),
            evm_old_address
        ));
        assert_eq!(
            DeeperNode::rewards_accounts_deeper_evm(1),
            Some(evm_old_address)
        );

        assert_ok!(DeeperNode::reward_mapping(
            Origin::signed(1),
            0,
            Vec::new(),
            evm_new_address
        ));
        assert_eq!(
            DeeperNode::rewards_accounts_deeper_evm(&1),
            Some(evm_new_address)
        );
    });
}

#[test]
fn reward_mapping_with_already_mapped_evm_address() {
    new_test_ext().execute_with(|| {
        let evm_address = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        assert_ok!(DeeperNode::reward_mapping(
            Origin::signed(1),
            0,
            Vec::new(),
            evm_address
        ));
        assert_eq!(
            DeeperNode::rewards_accounts_deeper_evm(&1),
            Some(evm_address)
        );

        assert_err!(
            DeeperNode::reward_mapping(Origin::signed(2), 0, Vec::new(), evm_address),
            Error::<Test>::EthAddressAlreadyMapped
        );
    });
}

#[test]
fn get_onboard_time() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        run_to_block(1);
        assert_eq!(DeeperNode::get_onboard_time(&1), Some(0));
    });
}

#[test]
fn im_ever_online() {
    new_test_ext().execute_with(|| {
        assert_eq!(DeeperNode::im_ever_online(&1), false);
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(DeeperNode::im_ever_online(&1), true);
    });
}

#[test]
fn get_eras_offline() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        run_to_block(BLOCKS_PER_ERA - 1);
        assert_eq!(DeeperNode::get_eras_offline(&1), 0);
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(DeeperNode::get_eras_offline(&1), 1);
        run_to_block(3 * BLOCKS_PER_ERA);
        assert_eq!(DeeperNode::get_eras_offline(&1), 3);
        assert_eq!(DeeperNode::get_eras_offline(&2), 3);
    });
}

#[test]
fn get_npow_reward() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        assert_err!(
            DeeperNode::get_npow_reward(Origin::signed(2)),
            Error::<Test>::NpowRewardAddressNotFound
        );

        assert_ok!(DeeperNode::reward_mapping(
            Origin::signed(1),
            0,
            Vec::new(),
            H160::zero(),
        ));
        assert_ok!(DeeperNode::get_npow_reward(Origin::signed(1)));
        assert_eq!(
            <frame_system::Pallet<Test>>::events()
                .pop()
                .expect("should contains events")
                .event,
            crate::tests::Event::from(crate::Event::GetNpowReward(1, H160::zero()))
        );
    });
}
