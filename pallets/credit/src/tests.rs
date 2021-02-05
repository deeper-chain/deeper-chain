use crate::mock::*;
use frame_support::assert_ok;
//use frame_system::ensure_signed;
use crate::CreditInterface;
use sp_runtime::DispatchError;

#[test]
fn fn_initialize_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // initialize credit [0-29]
        assert_ok!(Credit::initialize_credit(Origin::signed(1), 29));
        assert_eq!(Credit::get_user_credit(1), Some(0));

        assert_ok!(Credit::initialize_credit(Origin::signed(2), 0));
        assert_eq!(Credit::get_user_credit(2), Some(0));

        // initialize credit 30
        assert_eq!(
            Credit::initialize_credit(Origin::signed(3), 30),
            Err(DispatchError::Other("CreditInitFailed",))
        );

        // initialize credit twice
        assert_ok!(Credit::initialize_credit(Origin::signed(6), 29));
        assert_eq!(Credit::get_user_credit(6), Some(0));
        assert_eq!(
            Credit::initialize_credit(Origin::signed(6), 50),
            Err(DispatchError::Other("CreditInitFailed",))
        );
    });
}

#[test]
fn fn_kill_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // kill credit normal
        assert_ok!(Credit::initialize_credit(Origin::signed(1), 29));
        assert_ok!(Credit::kill_credit(Origin::signed(1)));

        // uninitialized account
        assert_eq!(
            Credit::kill_credit(Origin::signed(2)),
            Err(DispatchError::Other("KillCreditFailed",))
        );
    });
}

#[test]
fn fn_initialize_credit() {
    new_test_ext().execute_with(|| {
        // [0,29]
        assert_eq!(Credit::_initialize_credit(1, 0), true);
        assert_eq!(Credit::_initialize_credit(2, 29), true);

        // 30
        assert_eq!(Credit::_initialize_credit(3, 30), false);

        // initialize twice
        assert_eq!(Credit::_initialize_credit(4, 29), true);
        assert_eq!(Credit::_initialize_credit(4, 28), false);
    });
}

#[test]
fn fn_update_credit() {
    new_test_ext().execute_with(|| {
        // [30,100]
        let micropayment_vec = vec![(1, 5 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::get_user_credit(1), Some(5));
    });
}

#[test]
fn fn_attenuate_credit() {
    new_test_ext().execute_with(|| {
        // attenuate_credit successful
        let micropayment_vec = vec![(1, 50 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::get_user_credit(1), Some(50));
        assert_eq!(Credit::_attenuate_credit(1), true);
        assert_eq!(Credit::get_user_credit(1), Some(45));
        assert_eq!(Credit::_attenuate_credit(1), true);
        assert_eq!(Credit::get_user_credit(1), Some(40));
        assert_eq!(Credit::_attenuate_credit(1), false);

        // attenuate_credit failed
        let micropayment_vec2 = vec![(2, 30 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec2);
        assert_eq!(Credit::get_user_credit(2), Some(30));
        assert_eq!(Credit::_attenuate_credit(2), false);
        assert_eq!(Credit::get_user_credit(2), Some(30));
    });
}

#[test]
fn fn_kill_credit() {
    new_test_ext().execute_with(|| {
        // kill successfully
        assert_eq!(Credit::_initialize_credit(1, 29), true);
        assert_eq!(Credit::_kill_credit(1), true);

        // uninitialized account
        assert_eq!(Credit::_kill_credit(2), false);
    });
}

// CreditInterface test
#[test]
fn fn_get_credit_score() {
    new_test_ext().execute_with(|| {
        let micropayment_vec = vec![(1, 15 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);

        assert_eq!(Credit::get_credit_score(1), Some(15));
    });
}

#[test]
fn fn_pass_threshold() {
    new_test_ext().execute_with(|| {
        // <60
        let micropayment_vec = vec![(1, 55 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::pass_threshold(1, 0), false);

        // =60
        let micropayment_vec = vec![(2, 60 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::pass_threshold(2, 0), false);

        // >60
        let micropayment_vec = vec![(3, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::pass_threshold(3, 0), true);
    });
}

#[test]
fn fn_credit_slash() {
    new_test_ext().execute_with(|| {
        let micropayment_vec = vec![(1, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);

        Credit::credit_slash(1);
        assert_eq!(Credit::get_credit_score(1), Some(70));
    });
}
