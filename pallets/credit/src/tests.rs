use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::ensure_signed;
use sp_runtime::DispatchError;

#[test]
fn account_id_init_one_time() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        // Read pallet storage and assert an expected result.
        assert_eq!(Credit::get_user_credit(1), Some(50));
    });
}

#[test]
fn the_same_accountid_init_two_times() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        // Read pallet storage and assert an expected result.
        assert_eq!(Credit::get_user_credit(1), Some(50));

        // reinit the same account_id credit
        assert_eq!(
            Credit::initialize_credit_extrinsic(Origin::signed(1), 50),
            Err(DispatchError::Other(
                "CreditInitFailed",
            ))
        );
    });
}

#[test]
fn update_acccount_id_credit_score() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        // update_credit
        assert_ok!(Credit::update_credit_extrinsic(Origin::signed(1), 60));
    });
}

#[test]
fn update_uninit_acccount_id_credit_score() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        // update_credit
        assert_eq!(
            Credit::update_credit_extrinsic(Origin::signed(2), 60),
            Err(DispatchError::Other(
                "CreditUpdateFailed",
            ))
        );
    });
}

#[test]
fn delete_acccount_id_credit_score() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        // kill_credit
        assert_ok!(Credit::kill_credit_extrinsic(Origin::signed(1)));
    });
}

#[test]
fn delete_unexisted_account_id_credit_score() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Credit::initialize_credit_extrinsic(Origin::signed(1), 50));
        // kill_credit
        assert_eq!(
            Credit::kill_credit_extrinsic(Origin::signed(2)),
            Err(DispatchError::Other("KillCreditFailed",))
        );
    });
}

// CreditInterface Test
#[test]
fn attenuate_credit_test() {
    new_test_ext().execute_with(|| {
        // attenuate_credit successful
        let origin = Origin::signed(1);
        assert_ok!(Credit::initialize_credit_extrinsic(origin.clone(), 60));
        let account_id = ensure_signed(origin).unwrap_or_default();
        assert_eq!(Credit::attenuate_credit(account_id), true);

        // attenuate_credit failed
        let origin2 = Origin::signed(2);
        assert_ok!(Credit::initialize_credit_extrinsic(origin2.clone(), 30));
        let account_id = ensure_signed(origin2).unwrap_or_default();
        assert_eq!(Credit::attenuate_credit(account_id), false);
    });
}

/*
#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
           Credit::cause_error(Origin::signed(1)),
          Error::<Test>::NoneValue
        );
    });
}
*/
