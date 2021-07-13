use super::{CreditLevel, CreditSetting};
use crate::{mock::*, CreditInterface};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use sp_runtime::traits::BadOrigin;
use sp_runtime::Percent;

#[test]
fn fn_initialize_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // initialize credit [0-29]
        assert_ok!(Credit::initialize_credit(Origin::signed(1)));
        assert_eq!(Credit::get_user_credit(1), Some(60));
    });
}

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
            number_of_referees: 1,
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
        assert_eq!(
            Credit::get_credit_setting(CreditLevel::One).unwrap(),
            credit_setting
        );

        let credit_setting_updated = CreditSetting {
            credit_level: CreditLevel::One,
            balance: 40_000,
            base_apy: Percent::from_percent(45),
            bonus_apy: Percent::from_percent(3),
            tax_rate: Percent::from_percent(9),
            number_of_referees: 2,
            reward_per_referee: 18,
        };
        assert_ok!(Credit::update_credit_setting(
            RawOrigin::Root.into(),
            credit_setting_updated.clone()
        ));
        assert_eq!(
            Credit::get_credit_setting(CreditLevel::One).unwrap(),
            credit_setting_updated
        );
    });
}
