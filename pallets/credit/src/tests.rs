use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::ensure_signed;
use sp_runtime::DispatchError;

#[test]
fn fn_initialize_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // initialize credit [30-100]
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        assert_eq!(Credit::get_user_credit(1), Some(50));

        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(2), 30));
        assert_eq!(Credit::get_user_credit(2), Some(30));

        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(3), 100));
        assert_eq!(Credit::get_user_credit(3), Some(100));

        // initialize credit 20
        assert_eq!(
            Credit::initialize_credit_extrinsic(Origin::signed(4), 20),
            Err(DispatchError::Other("CreditInitFailed",))
        );

        // initialize credit 101
        assert_eq!(
            Credit::initialize_credit_extrinsic(Origin::signed(5), 101),
            Err(DispatchError::Other("CreditInitFailed",))
        );

        // initialize credit twice
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(6), 50));
        assert_eq!(Credit::get_user_credit(6), Some(50));
        assert_eq!(
            Credit::initialize_credit_extrinsic(Origin::signed(6), 50),
            Err(DispatchError::Other("CreditInitFailed",))
        );
    });
}

#[test]
fn fn_update_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // update after initialize
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        assert_ok!(Credit::update_credit_extrinsic(Origin::signed(1), 30));
        assert_ok!(Credit::update_credit_extrinsic(Origin::signed(1), 70));
        assert_ok!(Credit::update_credit_extrinsic(Origin::signed(1), 100));

        // update uninitialize
        assert_ok!(Credit::update_credit_extrinsic(Origin::signed(2), 60));

        // < 30
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(3), 50));
        assert_eq!(
            Credit::update_credit_extrinsic(Origin::signed(3), 29),
            Err(DispatchError::Other("CreditUpdateFailed",))
        );

        // >100
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(4), 50));
        assert_ok!(Credit::update_credit_extrinsic(Origin::signed(4), 105));
    });
}

#[test]
fn fn_kill_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // kill credit normal
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        assert_ok!(Credit::kill_credit_extrinsic(Origin::signed(1)));

        // uninitialized account
        assert_eq!(
            Credit::kill_credit_extrinsic(Origin::signed(2)),
            Err(DispatchError::Other("KillCreditFailed",))
        );
    });
}

#[test]
fn fn_initialize_credit() {
    new_test_ext().execute_with(|| {
        // [30,100]
        assert_eq!(Credit::initialize_credit(1, 30), true);
        assert_eq!(Credit::initialize_credit(2, 50), true);
        assert_eq!(Credit::initialize_credit(3, 100), true);

        // < 30
        assert_eq!(Credit::initialize_credit(4, 20), false);

        // > 100
        assert_eq!(Credit::initialize_credit(5, 101), false);

        // initialize twice
        assert_eq!(Credit::initialize_credit(6, 88), true);
        assert_eq!(Credit::initialize_credit(6, 88), false);
    });
}

#[test]
fn fn_update_credit() {
    new_test_ext().execute_with(|| {
        // [30,100]
        assert_eq!(Credit::initialize_credit(1, 50), true);
        assert_eq!(Credit::update_credit(1, 30), true);
        assert_eq!(Credit::update_credit(1, 100), true);

        // < 30
        assert_eq!(Credit::update_credit(1, 20), false);

        // > 100
        assert_eq!(Credit::update_credit(1, 101), true);

        // update uninitialize accout
        assert_eq!(Credit::update_credit(2, 88), true);
    });
}

#[test]
fn fn_attenuate_credit() {
    new_test_ext().execute_with(|| {
        // attenuate_credit successful
        assert_eq!(Credit::initialize_credit(1, 50), true);
        assert_eq!(Credit::attenuate_credit(1), true);
        assert_eq!(Credit::get_user_credit(1), Some(45));
        assert_eq!(Credit::attenuate_credit(1), true);
        assert_eq!(Credit::get_user_credit(1), Some(40));
        assert_eq!(Credit::attenuate_credit(1), false);

        // attenuate_credit failed
        assert_eq!(Credit::initialize_credit(2, 30), true);
        assert_eq!(Credit::attenuate_credit(2), false);
        assert_eq!(Credit::get_user_credit(2), Some(30));
    });
}

#[test]
fn fn_kill_credit() {
    new_test_ext().execute_with(|| {
        // kill successfully
        assert_eq!(Credit::initialize_credit(1, 50), true);
        assert_eq!(Credit::kill_credit(1), true);

        // uninitialized account
        assert_eq!(Credit::kill_credit(2), false);
    });
}
