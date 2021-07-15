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
                number_of_referees: 1,
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
                credit: 100,
                number_of_referees: 1,
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
            (1, 1_000 * 1_000_000_000_000_000, 1),
            (2, 1 * 1_000_000_000_000_000, 3),
            (3, 1 * 1_000_000_000_000_000, 0),
        ];
        Credit::update_credit(micropayments);
        assert_eq!(Credit::get_user_credit(&1).unwrap().credit, 5);
        assert_eq!(Credit::get_user_credit(&2).unwrap().credit, 1);
        assert_eq!(Credit::get_user_credit(&3).unwrap().credit, 0);
        micropayments = vec![
            (1, 4 * 1_000_000_000_000_000, 1),
            (2, 2 * 1_000_000_000_000_000, 3),
            (4, 1_000_000_000_000_000 / 10, 2),
        ];
        Credit::update_credit(micropayments);
        assert_eq!(Credit::get_user_credit(&1).unwrap().credit, 9); // 5 + 4
        assert_eq!(Credit::get_user_credit(&2).unwrap().credit, 3); // 1 + 2
        assert_eq!(Credit::get_user_credit(&4).unwrap().credit, 0);
    });
}
