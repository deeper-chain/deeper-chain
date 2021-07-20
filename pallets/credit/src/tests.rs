use super::{CreditData, CreditLevel, CreditSetting, UserCredit};
use crate::{mock::*, CreditInterface};
use frame_support::{assert_noop, assert_ok};
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
            credit_level: CreditLevel::One,
            balance: 20_000,
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
        assert_eq!(Credit::get_credit_setting(CreditLevel::One), credit_setting);

        let credit_setting_updated = CreditSetting {
            credit_level: CreditLevel::One,
            balance: 40_000,
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
            Credit::get_credit_setting(CreditLevel::One),
            credit_setting_updated
        );
    });
}

#[test]
fn get_credit_score() {
    new_test_ext().execute_with(|| {
        UserCredit::<Test>::insert(
            1,
            CreditData {
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                expiration: BLOCKS_PER_ERA,
            },
        );
        assert_eq!(Credit::get_credit_score(&1).unwrap(), 100);
    });
}

#[test]
fn get_number_of_referees() {
    new_test_ext().execute_with(|| {
        UserCredit::<Test>::insert(
            1,
            CreditData {
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                expiration: BLOCKS_PER_ERA,
            },
        );
        assert_eq!(Credit::get_number_of_referees(&1).unwrap(), 1);
    });
}

#[test]
fn slash_credit() {
    new_test_ext().execute_with(|| {
        UserCredit::<Test>::insert(
            1,
            CreditData {
                credit: 100,
                initial_credit_level: CreditLevel::One,
                rank_in_initial_credit_level: 1u32,
                number_of_referees: 1,
                expiration: BLOCKS_PER_ERA,
            },
        );
        Credit::slash_credit(&1);
        assert_eq!(Credit::get_credit_score(&1).unwrap(), 95);
    });
}

#[test]
fn update_credit() {
    new_test_ext().execute_with(|| {
        let mut micropayments = vec![
            (1, 1_000 * 1_000_000_000_000_000),
            (2, 1 * 1_000_000_000_000_000),
            (3, 1 * 1_000_000_000_000_000),
        ];
        Credit::update_credit(micropayments);
        assert_eq!(Credit::get_user_credit(&1).unwrap().credit, 5);
        assert_eq!(Credit::get_user_credit(&2).unwrap().credit, 1);
        assert_eq!(Credit::get_user_credit(&3).unwrap().credit, 101);
        micropayments = vec![
            (1, 4 * 1_000_000_000_000_000),
            (2, 2 * 1_000_000_000_000_000),
            (4, 1_000_000_000_000_000 / 10),
        ];
        Credit::update_credit(micropayments);
        assert_eq!(Credit::get_user_credit(&1).unwrap().credit, 9); // 5 + 4
        assert_eq!(Credit::get_user_credit(&2).unwrap().credit, 3); // 1 + 2
        assert_eq!(Credit::get_user_credit(&4).unwrap().credit, 0);
    });
}

#[test]
fn get_reward_work() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        assert_eq!(
            Credit::get_reward(&3),
            Some((18000000000000000000, 3369858941948251800))
        );
        run_to_block(BLOCKS_PER_ERA);
        assert_eq!(
            Credit::get_reward(&3),
            Some((18000000000000000000, 3369858941948251800))
        );
        run_to_block(BLOCKS_PER_ERA + 1);
        assert_eq!(Credit::get_reward(&3), Some((0, 3369858941948251800)));
    });
}

#[test]
fn get_reward_with_update_credit_no_bonus() {
    new_test_ext().execute_with(|| {
        assert_eq!(Credit::get_user_credit(&6).unwrap().credit, 100);
        assert_eq!(
            Credit::get_reward(&6),
            Some((18000000000000000000, 3369858941948251800))
        );

        let micropayments = vec![(6, 5 * 1_000_000_000_000_000)];
        let mut i = 1;
        while i < 20 {
            // run 19 times
            run_to_block(BLOCKS_PER_ERA * i + 1);

            Credit::update_credit(micropayments.clone());
            assert_eq!(Credit::get_user_credit(&6).unwrap().credit, 100 + 5 * i);
            assert_eq!(
                Credit::get_reward(&6),
                Some((18000000000000000000, 3369858941948251800))
            );
            i += 1;
        }

        run_to_block(BLOCKS_PER_ERA * 20 + 1);
        Credit::update_credit(micropayments.clone());
        assert_eq!(Credit::get_user_credit(&6).unwrap().credit, 100 + 5 * 20);
        assert_eq!(
            Credit::get_reward(&6),
            Some((18000000000000000000, 15287661460675804320))
        );
    });
}

#[test]
fn get_reward_with_update_credit_with_bonus() {
    new_test_ext().execute_with(|| {
        assert_eq!(Credit::get_user_credit(&7).unwrap().credit, 400);
        assert_eq!(
            Credit::get_reward(&7),
            Some((18000000000000000000 * 7, 97068450647875213020))
        );

        let micropayments = vec![(7, 5 * 1_000_000_000_000_000)];
        let mut i = 1;
        while i < 20 {
            // run 19 times
            run_to_block(BLOCKS_PER_ERA * i + 1);

            Credit::update_credit(micropayments.clone());
            assert_eq!(Credit::get_user_credit(&7).unwrap().credit, 400 + 5 * i);
            assert_eq!(
                Credit::get_reward(&7),
                Some((18000000000000000000 * 7, 97068450647875213020))
            );
            i += 1;
        }

        run_to_block(BLOCKS_PER_ERA * 20 + 1);
        Credit::update_credit(micropayments.clone());
        assert_eq!(Credit::get_user_credit(&7).unwrap().credit, 400 + 5 * 20);
        assert_eq!(
            Credit::get_reward(&7),
            Some((18000000000000000000 * 7, 131780755652680908140))
        );
    });
}

#[test]
fn get_reward_with_slash_credit_with_bonus() {
    new_test_ext().execute_with(|| {
        assert_eq!(Credit::get_user_credit(&7).unwrap().credit, 400);
        assert_eq!(
            Credit::get_reward(&7),
            Some((18000000000000000000 * 7, 97068450647875213020))
        );

        Credit::slash_credit(&7);
        assert_eq!(Credit::get_user_credit(&7).unwrap().credit, 400 - 5);
        assert_eq!(
            Credit::get_reward(&7),
            Some((18000000000000000000 * 7, 83523261467953134276))
        );
    });
}

#[test]
fn get_reward_failed() {
    new_test_ext().execute_with(|| {
        run_to_block(1);
        assert_eq!(Credit::get_reward(&5), None); // 5 credit 0
        assert_eq!(Credit::get_reward(&8), None); // 8 not contains in storage
    });
}
