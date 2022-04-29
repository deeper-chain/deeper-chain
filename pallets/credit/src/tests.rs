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

#[cfg(test)]


use super::{CreditData, CreditLevel, CreditSetting, UserCredit};
use crate::{mock::*, CampaignIdSwitch, CreditInterface, Error, UserCreditHistory};
use frame_support::traits::Currency;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError},
};
use frame_system::RawOrigin;
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
            Err(DispatchError::from(Error::<Test>::InvalidCreditData))
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
            Err(DispatchError::from(Error::<Test>::InvalidCreditData))
        );
    });
}

#[test]
fn add_or_update_credit_data_check_credit_history_and_reward() {
    new_test_ext().execute_with(|| {
        // era 0
        assert_ok!(DeeperNode::im_online(Origin::signed(3)));
        // era 1
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(Credit::user_credit_history(3), vec![]);
        assert!(Credit::init_delegator_history(&3, 0));
        assert_eq!(
            Credit::get_reward(&3, 0, 0).0,
            Some((0, 21369858941948251800))
        );
        let credit_historys = vec![(
            0,
            CreditData {
                campaign_id: 0,
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1,
                number_of_referees: 1,
                current_credit_level: CreditLevel::One,
                reward_eras: 1,
            },
        )];
        assert_eq!(Credit::user_credit_history(3), credit_historys);

        let credit_data = CreditData {
            campaign_id: 0,
            credit: 400,
            initial_credit_level: CreditLevel::Four,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::Four,
            reward_eras: 270,
        };

        assert_ok!(Credit::add_or_update_credit_data(
            RawOrigin::Root.into(),
            3,
            credit_data.clone()
        ));
        assert_eq!(Credit::user_credit(3), Some(credit_data));
        let credit_historys = vec![
            (
                0,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 1,
                },
            ),
            (
                1,
                CreditData {
                    campaign_id: 0,
                    credit: 400,
                    initial_credit_level: CreditLevel::Four,
                    rank_in_initial_credit_level: 0,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::Four,
                    reward_eras: 270,
                },
            ),
        ];
        assert_eq!(Credit::user_credit_history(3), credit_historys);

        // era 2
        run_to_block(BLOCKS_PER_ERA * 2);
        assert_eq!(
            Credit::get_reward(&3, 1, 1).0,
            Some((0, 223068450647875213020))
        );

        // era 3
        run_to_block(BLOCKS_PER_ERA * 3);
        assert_eq!(
            Credit::get_reward(&3, 2, 2).0,
            Some((0, 223068450647875213020))
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
        Credit::slash_credit(&1, None);
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
fn update_credit_by_tip() {
    new_test_ext().execute_with(|| {
        Credit::update_credit_by_tip(1, 8);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 0);

        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        Credit::update_credit_by_tip(1, 8);
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 8); // 0 + 8
    });
}

#[test]
fn update_credit_by_burn_nft() {
    new_test_ext().execute_with(|| {
        assert_ok!(Credit::update_credit_by_burn_nft(1, 8));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 8); // 0 + 8
    });
}

#[test]
fn get_reward_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Credit::get_reward(&3, 0, 0).0, None);
        assert!(Credit::init_delegator_history(&3, 0));
        assert!(Credit::init_delegator_history(&7, 0));
        assert!(Credit::init_delegator_history(&8, 0));
        assert!(Credit::init_delegator_history(&9, 0));
        assert!(Credit::init_delegator_history(&10, 0));
        assert!(Credit::init_delegator_history(&11, 0));
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
        Timestamp::set_timestamp(INIT_TIMESTAMP);
        assert_ok!(DeeperNode::im_online(Origin::signed(6)));
        assert_eq!(Credit::user_credit(&6).unwrap().credit, 100);
        assert!(Credit::init_delegator_history(&6, 0));
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
        assert!(Credit::init_delegator_history(&7, 0));
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
        assert_eq!(Credit::user_credit(&7).unwrap().credit, 400);
        assert!(Credit::init_delegator_history(&7, 0));
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(
            Credit::get_reward(&7, 0, 0).0,
            Some((0, 223068450647875213020))
        );

        Credit::slash_credit(&7, None);
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
fn update_credit_history_when_era_is_the_same() {
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
                reward_eras: 270,
            },
        );
        assert!(Credit::init_delegator_history(&1, 0));
        //default era=0

        let credit_historys = vec![(
            0,
            CreditData {
                campaign_id: 0,
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                current_credit_level: CreditLevel::One,
                reward_eras: 270,
            },
        )];

        assert_eq!(Credit::user_credit_history(1), credit_historys);
    });
}

#[test]
fn update_credit_history_when_era_is_non_zero() {
    new_test_ext().execute_with(|| {
        //default era = 0
        UserCredit::<Test>::insert(
            1,
            CreditData {
                campaign_id: 0,
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                current_credit_level: CreditLevel::One,
                reward_eras: 270,
            },
        );
        // run_to_block, era=1
        run_to_block(BLOCKS_PER_ERA);
        assert!(Credit::init_delegator_history(&1, 1));
        Credit::update_credit_history(&1, 10);

        let credit_historys = vec![
            (
                1,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 270,
                },
            ),
            (
                10,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 270,
                },
            ),
        ];

        assert_eq!(Credit::user_credit_history(1), credit_historys);
    });
}

#[test]
fn burn_dpr_add_credit() {
    new_test_ext().execute_with(|| {
        // 1,3's gennesis balance = 500
        let _ = Balances::deposit_creating(&1, 5000);
        let _ = Balances::deposit_creating(&3, 10000);
        // genesis 1's credit score 100
        UserCreditHistory::<Test>::insert(
            1,
            vec![
                (
                    1,
                    CreditData {
                        campaign_id: 0,
                        credit: 10,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
                (
                    2,
                    CreditData {
                        campaign_id: 0,
                        credit: 50,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
            ],
        );

        // genesis 3's credit score 100
        UserCreditHistory::<Test>::insert(
            3,
            vec![
                (
                    1,
                    CreditData {
                        campaign_id: 0,
                        credit: 100,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
                (
                    2,
                    CreditData {
                        campaign_id: 0,
                        credit: 300,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::Three,
                        reward_eras: 270,
                    },
                ),
            ],
        );
        // run_to_block, era=1
        run_to_block(BLOCKS_PER_ERA * 3);

        assert!(Credit::burn_for_add_credit(Origin::signed(1), 100).is_ok());
        assert!(Credit::burn_for_add_credit(Origin::signed(3), 200).is_ok());

        assert_eq!(Credit::user_credit(1).unwrap().credit, 100);
        assert_eq!(Credit::user_credit(3).unwrap().credit, 300);

        assert_eq!(Balances::free_balance(&1), 500);
        assert_eq!(Balances::free_balance(&3), 500);
        assert_eq!(Treasury::pot(), 10000 + 5000);
    });
}

#[test]
fn switch_campaign_duration() {
    new_test_ext().execute_with(|| {
        // 1,3's gennesis balance = 500
        let _ = Balances::deposit_creating(&13, 5000);
        CampaignIdSwitch::<Test>::insert(0, 1);
        //let credit_data = Credit::user_credit(1).unwrap();
        assert!(Credit::init_delegator_history(&13, 1));
        // run_to_block, era=1
        run_to_block(BLOCKS_PER_ERA * 2);
        assert_eq!(
            Credit::get_reward(&13, 1, 1).0,
            Some((0, 60263002216294070076))
        );

        run_to_block(BLOCKS_PER_ERA * 3);
        assert_eq!(
            Credit::get_reward(&13, 2, 2).0,
            Some((0, 56416427606743384752))
        );

        assert_eq!(Credit::user_credit(13).unwrap().campaign_id, 1);
        assert_eq!(Credit::user_credit(13).unwrap().reward_eras, 1 + 180);

        run_to_block(BLOCKS_PER_ERA * 4);
        assert_eq!(
            Credit::get_reward(&13, 3, 3).0,
            Some((0, 56416427606743384752))
        );
    });
}

#[test]
fn force_modify_credit_history() {
    new_test_ext().execute_with(|| {
        UserCreditHistory::<Test>::insert(
            1,
            vec![
                (
                    6,
                    CreditData {
                        campaign_id: 0,
                        credit: 110,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
                (
                    10,
                    CreditData {
                        campaign_id: 0,
                        credit: 109,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
            ],
        );
        assert!(Credit::force_modify_credit_history(Origin::root().into(), 1, 8).is_ok());
        assert_eq!(
            Credit::user_credit_history(1),
            vec![
                (
                    8,
                    CreditData {
                        campaign_id: 0,
                        credit: 110,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
                (
                    10,
                    CreditData {
                        campaign_id: 0,
                        credit: 109,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
            ]
        );

        assert!(Credit::force_modify_credit_history(Origin::root().into(), 1, 6).is_err()); // do not modify
        assert_eq!(
            Credit::user_credit_history(1),
            vec![
                (
                    8,
                    CreditData {
                        campaign_id: 0,
                        credit: 110,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
                (
                    10,
                    CreditData {
                        campaign_id: 0,
                        credit: 109,
                        initial_credit_level: CreditLevel::One,
                        rank_in_initial_credit_level: 1u32,
                        number_of_referees: 1,
                        current_credit_level: CreditLevel::One,
                        reward_eras: 270,
                    },
                ),
            ]
        );

        assert!(Credit::force_modify_credit_history(Origin::root().into(), 1, 10).is_ok());
        assert_eq!(
            Credit::user_credit_history(1),
            vec![(
                10,
                CreditData {
                    campaign_id: 0,
                    credit: 109,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 270,
                },
            )]
        );

        assert!(Credit::force_modify_credit_history(Origin::root().into(), 1, 12).is_ok());
        assert_eq!(
            Credit::user_credit_history(1),
            vec![(
                12,
                CreditData {
                    campaign_id: 0,
                    credit: 109,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 270,
                },
            )]
        );

        assert!(Credit::force_modify_credit_history(Origin::root().into(), 1, 12).is_ok());
        assert_eq!(
            Credit::user_credit_history(1),
            vec![(
                12,
                CreditData {
                    campaign_id: 0,
                    credit: 109,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 270,
                },
            )]
        );
    });
}

#[test]
fn update_nft_class_credit() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Credit::update_nft_class_credit(Origin::signed(1), 0, 5),
            BadOrigin
        );

        assert_ok!(Credit::update_nft_class_credit(Origin::root(), 0, 5));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(0), 5);

        assert_ok!(Credit::update_nft_class_credit(Origin::root(), 1, 10));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(1), 10);
    });
}

#[test]
fn burn_nft() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::force_create(Origin::root(), 0, 1, true));
        assert_ok!(Uniques::force_create(Origin::root(), 1, 1, true));
        assert_ok!(Uniques::force_create(Origin::root(), 2, 1, true));

        assert_ok!(Uniques::mint(Origin::signed(1), 0, 42, 1));
        assert_ok!(Uniques::mint(Origin::signed(1), 1, 42, 1));
        assert_ok!(Uniques::mint(Origin::signed(1), 2, 42, 1));

        assert_noop!(
            Credit::burn_nft(Origin::signed(1), 0, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );
        assert_noop!(
            Credit::burn_nft(Origin::signed(1), 1, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );
        assert_noop!(
            Credit::burn_nft(Origin::signed(1), 2, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );

        assert_ok!(Credit::update_nft_class_credit(Origin::root(), 0, 5));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(0), 5);
        assert_ok!(Credit::update_nft_class_credit(Origin::root(), 1, 10));
        assert_eq!(crate::MiningMachineClassCredit::<Test>::get(1), 10);

        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        // update_credit_data works
        assert_ok!(Credit::add_or_update_credit_data(
            Origin::root(),
            1,
            credit_data.clone()
        ));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 100);

        assert_ok!(Credit::burn_nft(Origin::signed(1), 0, 42));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 105);

        assert_ok!(Credit::burn_nft(Origin::signed(1), 1, 42));
        assert_eq!(Credit::user_credit(&1).unwrap().credit, 115);

        assert_noop!(
            Credit::burn_nft(Origin::signed(1), 2, 42),
            Error::<Test>::MiningMachineClassCreditNoConfig
        );
    });
}
