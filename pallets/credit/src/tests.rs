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

use super::{CampaignData, CampaignStatus, CreditData, CreditLevel, CreditSetting, UserCredit};
use crate::{mock::*, CreditInterface, Error};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo};
use frame_system::RawOrigin;
use pallet_balances::Error as BalancesError;
use sp_runtime::traits::BadOrigin;
use sp_runtime::Percent;

#[test]
fn get_credit_level() {
    new_test_ext().execute_with(|| {
        assert_eq!(Credit::get_credit_level(0), CreditLevel::Zero);
        assert_eq!(Credit::get_credit_level(50), CreditLevel::Zero);
        assert_eq!(Credit::get_credit_level(99), CreditLevel::Zero);
        assert_eq!(Credit::get_credit_level(100), CreditLevel::One);
        assert_eq!(Credit::get_credit_level(150), CreditLevel::One);
        assert_eq!(Credit::get_credit_level(199), CreditLevel::One);
        assert_eq!(Credit::get_credit_level(200), CreditLevel::Two);
        assert_eq!(Credit::get_credit_level(250), CreditLevel::Two);
        assert_eq!(Credit::get_credit_level(299), CreditLevel::Two);
        assert_eq!(Credit::get_credit_level(300), CreditLevel::Three);
        assert_eq!(Credit::get_credit_level(350), CreditLevel::Three);
        assert_eq!(Credit::get_credit_level(399), CreditLevel::Three);
        assert_eq!(Credit::get_credit_level(400), CreditLevel::Four);
        assert_eq!(Credit::get_credit_level(450), CreditLevel::Four);
        assert_eq!(Credit::get_credit_level(499), CreditLevel::Four);
        assert_eq!(Credit::get_credit_level(500), CreditLevel::Five);
        assert_eq!(Credit::get_credit_level(550), CreditLevel::Five);
        assert_eq!(Credit::get_credit_level(599), CreditLevel::Five);
        assert_eq!(Credit::get_credit_level(600), CreditLevel::Six);
        assert_eq!(Credit::get_credit_level(650), CreditLevel::Six);
        assert_eq!(Credit::get_credit_level(699), CreditLevel::Six);
        assert_eq!(Credit::get_credit_level(700), CreditLevel::Seven);
        assert_eq!(Credit::get_credit_level(750), CreditLevel::Seven);
        assert_eq!(Credit::get_credit_level(799), CreditLevel::Seven);
        assert_eq!(Credit::get_credit_level(800), CreditLevel::Eight);
        assert_eq!(Credit::get_credit_level(950), CreditLevel::Eight);
        assert_eq!(Credit::get_credit_level(1099), CreditLevel::Eight);
    });
}

#[test]
fn update_credit_setting() {
    new_test_ext().execute_with(|| {
        let credit_setting = CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::One,
            staking_balance: 20_000,
            base_apy: Percent::from_percent(39),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 1u32,
            tax_rate: Percent::from_percent(10),
            max_referees_with_rewards: 1,
            reward_per_referee: 18,
        };
        assert_noop!(
            Credit::update_credit_setting(Origin::signed(1), credit_setting.clone()),
            BadOrigin
        );
        assert_ok!(Credit::update_credit_setting(
            RawOrigin::Root.into(),
            credit_setting.clone()
        ));
        assert_eq!(Credit::credit_settings(0, CreditLevel::One), credit_setting);

        let credit_setting_updated = CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::One,
            staking_balance: 40_000,
            base_apy: Percent::from_percent(45),
            bonus_apy: Percent::from_percent(3),
            max_rank_with_bonus: 2u32,
            tax_rate: Percent::from_percent(9),
            max_referees_with_rewards: 2,
            reward_per_referee: 18,
        };
        assert_ok!(Credit::update_credit_setting(
            RawOrigin::Root.into(),
            credit_setting_updated.clone()
        ));
        assert_eq!(
            Credit::credit_settings(0, CreditLevel::One),
            credit_setting_updated
        );
    });
}

#[test]
fn add_or_update_credit_data() {
    new_test_ext().execute_with(|| {
        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        // Only sudo can call update_credit_data
        assert_noop!(
            Credit::add_or_update_credit_data(Origin::signed(1), 2, credit_data.clone()),
            BadOrigin
        );

        // update_credit_data works
        assert_ok!(Credit::add_or_update_credit_data(
            RawOrigin::Root.into(),
            1,
            credit_data.clone()
        ));
        assert_eq!(Credit::user_credit(1), Some(credit_data));

        // credit_data invalid
        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::Two,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::Two,
            reward_eras: 0,
        };
        assert_eq!(
            Credit::add_or_update_credit_data(RawOrigin::Root.into(), 1, credit_data.clone()),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::InvalidCreditData
            ))
        );

        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 10,
            current_credit_level: CreditLevel::One,
            reward_eras: 100,
        };
        assert_eq!(
            Credit::add_or_update_credit_data(RawOrigin::Root.into(), 1, credit_data.clone()),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::InvalidCreditData
            ))
        );
    });
}

#[test]
fn get_credit_score() {
    new_test_ext().execute_with(|| {
        UserCredit::<Test>::insert(
            1,
            CreditData {
                campaign_id: 0,
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                current_credit_level: CreditLevel::One,
                reward_eras: 1,
            },
        );
        assert_eq!(Credit::get_credit_score(&1).unwrap(), 100);
    });
}

#[test]
fn slash_credit() {
    new_test_ext().execute_with(|| {
        UserCredit::<Test>::insert(
            1,
            CreditData {
                campaign_id: 0,
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                reward_eras: 1,
                current_credit_level: CreditLevel::One,
            },
        );
        Credit::slash_credit(&1);
        assert_eq!(
            Credit::get_credit_score(&1).unwrap(),
            100 - CREDIT_ATTENUATION_STEP
        );
    });
}

#[test]
fn update_credit() {
    new_test_ext().execute_with(|| {
        Credit::update_credit((1, 1_000 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 0);

        for i in 1..5 {
            assert_ok!(DeeperNode::im_online(Origin::signed(i)));
        }

        Credit::update_credit((1, 1_000 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 1);

        Credit::update_credit((2, 1 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&2).unwrap().credit, 1);

        Credit::update_credit((3, 1 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 101);

        run_to_block(BLOCKS_PER_ERA * 2);
        Credit::update_credit((1, 4 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 2); // 1 + 1

        Credit::update_credit((2, 2 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&2).unwrap().credit, 2); // 1 + 1

        Credit::update_credit((4, 1_000_000_000_000_000 / 10));
        assert_eq!(Credit::user_credit(&4).unwrap().credit, 0);
    });
}

#[test]
fn update_credit_by_traffic() {
    new_test_ext().execute_with(|| {
        Credit::update_credit_by_traffic(1);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 0);

        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        Credit::update_credit_by_traffic(1);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 0);

        run_to_block(BLOCKS_PER_ERA * 2);
        Credit::update_credit_by_traffic(1);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 1); // 0 + 1

        run_to_block(BLOCKS_PER_ERA * 3);
        Credit::update_credit_by_traffic(1);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 1); // 1 + 0

        run_to_block(BLOCKS_PER_ERA * 4);
        Credit::update_credit_by_traffic(1);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 2); // 1 + 1
    });
}

#[test]
fn get_reward_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(3)));
        assert_ok!(DeeperNode::im_online(Origin::signed(7)));
        assert_ok!(DeeperNode::im_online(Origin::signed(8)));
        assert_ok!(DeeperNode::im_online(Origin::signed(9)));
        assert_ok!(DeeperNode::im_online(Origin::signed(10)));
        assert_ok!(DeeperNode::im_online(Origin::signed(11)));
        assert_eq!(Credit::get_reward(&3, 0, 0).0, None);
        run_to_block(BLOCKS_PER_ERA); // era 1
        assert_eq!(
            Credit::get_reward(&3, 0, 0).0,
            Some((0, 21369858941948251800))
        );
        assert_eq!(
            Credit::get_reward(&7, 0, 0).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&8, 0, 0).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&9, 0, 0).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&10, 0, 0).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&11, 0, 0).0,
            Some((0, 56416427606743384752))
        );
        run_to_block(BLOCKS_PER_ERA * 2); // era 2, credit expires at era 1
        assert_eq!(Credit::get_reward(&3, 1, 1).0, None);
        assert_eq!(
            Credit::get_reward(&7, 1, 1).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&8, 1, 1).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&9, 1, 1).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&10, 1, 1).0,
            Some((0, 223068450647875213020))
        );
        assert_eq!(
            Credit::get_reward(&11, 1, 1).0,
            Some((0, 56416427606743384752))
        );
    });
}

#[test]
fn get_reward_with_update_credit_no_bonus() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(6)));
        assert_eq!(Credit::user_credit(&6).unwrap().credit, 100);
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(
            Credit::get_reward(&6, 0, 0).0,
            Some((0, 21369858941948251800))
        );

        let mut i: u32 = 1;
        while i < 20 {
            // run 19 times
            run_to_block(BLOCKS_PER_ERA * i as u64 + 1);
            Credit::update_credit((6, 5 * 1_000_000_000_000_000));
            // to avoid slashing for being offline for 3 eras
            assert_ok!(DeeperNode::im_online(Origin::signed(6)));
            assert_eq!(
                Credit::user_credit(&6).unwrap().credit,
                100 + (i as u64 + 1) / 2
            );
            assert_eq!(
                Credit::get_reward(&6, i - 1, i - 1).0,
                Some((0, 21369858941948251800))
            );
            i += 1;
        }

        run_to_block(BLOCKS_PER_ERA * 200);
        Credit::update_credit((6, 190 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&6).unwrap().credit, 100 + 100);
        run_to_block(BLOCKS_PER_ERA * 201);
        assert_eq!(
            Credit::get_reward(&6, 200, 200).0,
            Some((0, 60263002216294070076))
        );
    });
}

#[test]
fn get_reward_with_update_credit_with_bonus() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(7)));
        assert_eq!(Credit::user_credit(&7).unwrap().credit, 400);
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(
            Credit::get_reward(&7, 0, 0).0,
            Some((0, 223068450647875213020))
        );

        let mut i: u32 = 1;
        while i < 20 {
            // run 19 times
            run_to_block(BLOCKS_PER_ERA * i as u64 + 1);
            Credit::update_credit((7, 5 * 1_000_000_000_000_000));
            // to avoid slashing for being offline for 3 eras
            assert_ok!(DeeperNode::im_online(Origin::signed(7)));
            assert_eq!(
                Credit::user_credit(&7).unwrap().credit,
                400 + (i as u64 + 1) / 2
            );
            assert_eq!(
                Credit::get_reward(&7, i - 1, i - 1).0,
                Some((0, 223068450647875213020))
            );
            i += 1;
        }

        run_to_block(BLOCKS_PER_ERA * 200);
        Credit::update_credit((7, 190 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&7).unwrap().credit, 400 + 100);
        run_to_block(BLOCKS_PER_ERA * 201);
        assert_eq!(
            Credit::get_reward(&7, 200, 200).0,
            Some((0, 394191705713783906280))
        );
    });
}

#[test]
fn get_reward_with_slash_credit_with_bonus() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(7)));
        assert_eq!(Credit::user_credit(&7).unwrap().credit, 400);
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(
            Credit::get_reward(&7, 0, 0).0,
            Some((0, 223068450647875213020))
        );

        Credit::slash_credit(&7);
        assert_eq!(
            Credit::user_credit(&7).unwrap().credit,
            400 - CREDIT_ATTENUATION_STEP
        );
        run_to_block(BLOCKS_PER_ERA * 2);
        assert_eq!(
            Credit::get_reward(&7, 1, 1).0,
            Some((0, 111517786970905338624))
        );
    });
}

#[test]
fn get_reward_failed() {
    new_test_ext().execute_with(|| {
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(Credit::get_reward(&5, 0, 0).0, None); // 5 credit 0
        assert_eq!(Credit::get_reward(&8, 0, 0).0, None); // 8 not contains in storage
    });
}

#[test]
fn slash_offline_devices_credit() {
    new_test_ext().execute_with(|| {
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 100);
        assert_ok!(DeeperNode::im_online(Origin::signed(3)));

        run_to_block(BLOCKS_PER_ERA);
        Credit::slash_offline_device_credit(&3);
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 100);

        run_to_block(BLOCKS_PER_ERA * 3);
        Credit::slash_offline_device_credit(&3);
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 99);

        run_to_block(BLOCKS_PER_ERA * 5);
        Credit::slash_offline_device_credit(&3);
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 99);

        run_to_block(BLOCKS_PER_ERA * 6);
        Credit::slash_offline_device_credit(&3);
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 98);

        run_to_block(BLOCKS_PER_ERA * 8);
        Credit::slash_offline_device_credit(&3);
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 98);

        run_to_block(BLOCKS_PER_ERA * 9);
        Credit::slash_offline_device_credit(&3);
        assert_eq!(Credit::user_credit(&3).unwrap().credit, 97);
    });
}

#[test]
fn set_campaign_data() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        // root
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        // not root
        assert_noop!(
            Credit::set_campaign_data(Origin::signed(1), campagin_data),
            BadOrigin
        );

        // multi stake_list
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![
                (2000, 400u64, CreditLevel::Four),
                (3000, 500u64, CreditLevel::Five),
                (1000, 100u64, CreditLevel::One),
            ],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        let campaign_data1 = Credit::campaign_datas(3).unwrap();
        assert_eq!(
            campaign_data1.stake_list,
            vec![
                (1000, 100u64, CreditLevel::One),
                (2000, 400u64, CreditLevel::Four),
                (3000, 500u64, CreditLevel::Five)
            ]
        );

        //campagin data error
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Six)],
        };
        assert_eq!(
            Credit::set_campaign_data(RawOrigin::Root.into(), campagin_data),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::CampaignDataError
            ))
        );
    });
}

#[test]
fn restart_campaign() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Stop,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        // root: Stop-> Start
        assert_ok!(Credit::restart_campaign(RawOrigin::Root.into(), 3));
        // not root
        assert_noop!(Credit::restart_campaign(Origin::signed(1), 3), BadOrigin);

        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        // root: Start-> Start
        assert_eq!(
            Credit::restart_campaign(RawOrigin::Root.into(), 3),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::CampaignStatusStarted
            ))
        );
        // root: InvalidCampaignId
        assert_eq!(
            Credit::restart_campaign(RawOrigin::Root.into(), 1),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::InvalidCampaignId
            ))
        );
    });
}

#[test]
fn stop_campaign() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        // root: Start-> Stop
        assert_ok!(Credit::stop_campaign(RawOrigin::Root.into(), 3));
        // not root
        assert_noop!(Credit::stop_campaign(Origin::signed(1), 3), BadOrigin);

        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Stop,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        // root: Stop-> Stop
        assert_eq!(
            Credit::stop_campaign(RawOrigin::Root.into(), 3),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::CampaignStatusStoped
            ))
        );
        // root: InvalidCampaignId
        assert_eq!(
            Credit::stop_campaign(RawOrigin::Root.into(), 1),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::InvalidCampaignId
            ))
        );
    });
}

#[test]
fn construct_credit_data() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));

        let credit_data = CreditData {
            campaign_id: 3,
            credit: 400,
            initial_credit_level: CreditLevel::Four,
            rank_in_initial_credit_level: 0,
            number_of_referees: 0,
            current_credit_level: CreditLevel::Four,
            reward_eras: 270,
        };
        assert_eq!(
            Credit::construct_credit_data(1000, campagin_data.clone()),
            None
        );
        assert_eq!(
            Credit::construct_credit_data(2000, campagin_data.clone()),
            Some(credit_data.clone())
        );
        assert_eq!(
            Credit::construct_credit_data(3000, campagin_data),
            Some(credit_data)
        );

        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![
                (2000, 400u64, CreditLevel::Four),
                (4000, 500u64, CreditLevel::Five),
            ],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));

        let credit_data = CreditData {
            campaign_id: 3,
            credit: 400,
            initial_credit_level: CreditLevel::Four,
            rank_in_initial_credit_level: 0,
            number_of_referees: 0,
            current_credit_level: CreditLevel::Four,
            reward_eras: 270,
        };
        assert_eq!(
            Credit::construct_credit_data(2000, campagin_data.clone()),
            Some(credit_data.clone())
        );
        assert_eq!(
            Credit::construct_credit_data(3000, campagin_data.clone()),
            Some(credit_data)
        );

        let credit_data = CreditData {
            campaign_id: 3,
            credit: 500,
            initial_credit_level: CreditLevel::Five,
            rank_in_initial_credit_level: 0,
            number_of_referees: 0,
            current_credit_level: CreditLevel::Five,
            reward_eras: 270,
        };
        assert_eq!(
            Credit::construct_credit_data(4000, campagin_data.clone()),
            Some(credit_data.clone())
        );
        assert_eq!(
            Credit::construct_credit_data(5000, campagin_data),
            Some(credit_data)
        );
    });
}

#[test]
fn stake_use_new_account() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        assert_eq!(Balances::free_balance(12), 8000);
        assert_ok!(Credit::stake(Origin::signed(12), 3, 2000));
        assert_eq!(Balances::reserved_balance(12), 2000);

        assert_eq!(
            Credit::stake(Origin::signed(12), 2, 2000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::InvalidCampaignId
            ))
        );
        assert_eq!(
            Credit::stake(Origin::signed(13), 3, 1000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakeBalanceTooLow
            ))
        );
        assert_eq!(
            Credit::stake(Origin::signed(13), 3, 2000),
            Err(DispatchErrorWithPostInfo::from(
                BalancesError::<Test, _>::InsufficientBalance
            ))
        );
    });
}

#[test]
fn stake_use_new_account_and_upgrade() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: true,
            stake_list: vec![
                (2000, 400u64, CreditLevel::Four),
                (4000, 500u64, CreditLevel::Five),
            ],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        // upgrade work
        assert_eq!(Balances::free_balance(12), 8000);
        assert_ok!(Credit::stake(Origin::signed(12), 3, 2000));
        assert_eq!(Balances::reserved_balance(12), 2000);

        assert_ok!(Credit::stake(Origin::signed(12), 3, 4000));
        assert_eq!(Balances::reserved_balance(12), 4000);

        // Onboarded
        assert_eq!(Balances::free_balance(14), 8000);
        assert_ok!(Credit::stake(Origin::signed(14), 3, 2000));
        assert_eq!(Balances::reserved_balance(14), 2000);
        assert_ok!(DeeperNode::im_online(Origin::signed(14)));
        assert_eq!(
            Credit::stake(Origin::signed(14), 3, 2000),
            Err(DispatchErrorWithPostInfo::from(Error::<Test>::Onboarded))
        );

        // InvalidUpgradeCreditLevel
        assert_eq!(Balances::free_balance(15), 8000);
        assert_ok!(Credit::stake(Origin::signed(15), 3, 4000));
        assert_eq!(Balances::reserved_balance(15), 4000);
        assert_eq!(
            Credit::stake(Origin::signed(15), 3, 2000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::InvalidUpgradeCreditLevel
            ))
        );
    });
}

#[test]
fn stake_to_another_campaign_when_expired() {
    new_test_ext().execute_with(|| {
        // stake one
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: true,
            stake_list: vec![
                (2000, 400u64, CreditLevel::Four),
                (4000, 500u64, CreditLevel::Five),
            ],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        assert_eq!(Balances::free_balance(12), 8000);
        assert_ok!(Credit::stake(Origin::signed(12), 3, 2000));
        assert_eq!(Balances::reserved_balance(12), 2000);
        assert_ok!(DeeperNode::im_online(Origin::signed(12)));

        // stake two
        let campagin_data = CampaignData {
            campaign_id: 4,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: true,
            stake_list: vec![
                (2000, 400u64, CreditLevel::Four),
                (4000, 500u64, CreditLevel::Five),
            ],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));

        run_to_block(BLOCKS_PER_ERA * 270 - 1);
        assert_eq!(
            Credit::stake(Origin::signed(12), 4, 2000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakedCampaignUnExpired
            ))
        );

        run_to_block(BLOCKS_PER_ERA * 270);
        Credit::update_credit((12, 190 * 1_000_000_000_000_000));
        assert_eq!(Credit::user_credit(&12).unwrap().credit, 400 + 270 / 2);
        assert_eq!(
            Credit::stake(Origin::signed(12), 4, 2000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakedOnAnotherCampaign
            ))
        );

        assert_ok!(Credit::unstake(Origin::signed(12)));
        let credit_data = CreditData {
            campaign_id: 3,
            credit: 135,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 0,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        assert_eq!(Credit::user_credit(12).unwrap_or_default(), credit_data);

        assert_ok!(Credit::stake(Origin::signed(12), 4, 4000));
        let credit_data = CreditData {
            campaign_id: 4,
            credit: 635,
            initial_credit_level: CreditLevel::Six,
            rank_in_initial_credit_level: 0,
            number_of_referees: 0,
            current_credit_level: CreditLevel::Six,
            reward_eras: 540,
        };
        assert_eq!(Credit::user_credit(12).unwrap_or_default(), credit_data);
        assert_eq!(Balances::reserved_balance(12), 4000);

        // stake three
        run_to_block(BLOCKS_PER_ERA * 540 - 1);
        assert_eq!(
            Credit::stake(Origin::signed(12), 3, 2000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakedCampaignUnExpired
            ))
        );

        run_to_block(BLOCKS_PER_ERA * 540);
        assert_eq!(
            Credit::stake(Origin::signed(12), 3, 2000),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakedOnAnotherCampaign
            ))
        );

        assert_ok!(Credit::unstake(Origin::signed(12)));
        assert_ok!(Credit::stake(Origin::signed(12), 3, 2000));
        assert_eq!(Balances::reserved_balance(12), 2000);
    });
}

#[test]
fn unstake_staked_account() {
    new_test_ext().execute_with(|| {
        let campagin_data = CampaignData {
            campaign_id: 3,
            campaign_status: CampaignStatus::Start,
            reward_eras: 270,
            upgradable: false,
            stake_list: vec![(2000, 400u64, CreditLevel::Four)],
        };
        assert_ok!(Credit::set_campaign_data(
            RawOrigin::Root.into(),
            campagin_data.clone()
        ));
        assert_eq!(Balances::free_balance(12), 8000);
        assert_ok!(Credit::stake(Origin::signed(12), 3, 2000));
        assert_eq!(Balances::reserved_balance(12), 2000);

        assert_eq!(
            Credit::unstake(Origin::signed(12)),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakedCampaignUnExpired
            ))
        );

        assert_ok!(DeeperNode::im_online(Origin::signed(12)));
        run_to_block(BLOCKS_PER_ERA * 270);
        assert_ok!(Credit::unstake(Origin::signed(12)));
        assert_eq!(Balances::reserved_balance(12), 0);
    });
}

#[test]
fn unstake_unstaked_account() {
    new_test_ext().execute_with(|| {
        let credit_data = CreditData {
            campaign_id: 0,
            credit: 203,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::Two,
            reward_eras: 270,
        };

        assert_ok!(Credit::add_or_update_credit_data(
            RawOrigin::Root.into(),
            12,
            credit_data.clone()
        ));

        assert_eq!(
            Credit::unstake(Origin::signed(12)),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::StakedCampaignUnExpired
            ))
        );

        assert_ok!(DeeperNode::im_online(Origin::signed(12)));
        run_to_block(BLOCKS_PER_ERA * 270);
        assert_ok!(Credit::unstake(Origin::signed(12)));

        let credit_data = CreditData {
            campaign_id: 0,
            credit: 103,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        assert_eq!(Credit::user_credit(12).unwrap_or_default(), credit_data);
    });
}
