use crate::{mock::*, Error, CreditDelegateInterface, CREDIT_LOCK_DURATION};
use frame_support::{assert_noop, assert_ok};
use pallet_credit::CreditInterface;

#[test]
fn test_delegate_credit_score(){
    new_test_ext().execute_with(|| {

        // initialize candidate list
        Delegating::set_candidate_validators(vec![4,6,8,10]);

        // initialize credit score
        Credit::initialize_credit(10,80);
        assert_eq!(Credit::get_credit_score(10),Some(80));

        // delegate credit score
        assert_ok!(Delegating::delegate(Origin::signed(10),8));

        // check delegated score
        let ledger = Delegating::credit_ledger(10);
        assert_eq!(Some(ledger.delegated_score), Credit::get_credit_score(10));

        // delegate with invalid validator
        assert_noop!(Delegating::delegate(Origin::signed(2),7), Error::<Test>::NonCandidateValidator);

    });
}

// #[test]
// fn delegate_duplicated(){
//     new_test_ext().execute_with(|| {
//         Delegating::set_candidate_validators(vec![4,6,8,10]);
//         Credit::initialize_credit(1,80);
//         assert_eq!(Credit::get_credit_score(1),Some(80));
//
//         assert_ok!(Delegating::delegate(Origin::signed(1),8));
//
//         // delegating duplicated
//         assert_noop!(Delegating::delegate(Origin::signed(1),8),Error::<Test>::AlreadyDelegated);
//
//     });
// }

#[test]
fn test_delegate_use_weak_credit(){
    new_test_ext().execute_with(|| {

        Delegating::set_candidate_validators(vec![4,6,8,10]);
        assert_eq!(Credit::initialize_credit(1,35),true);
        assert_eq!(Credit::get_credit_score(1),Some(35));

        // delegate with weak credit
        assert_noop!(Delegating::delegate(Origin::signed(1),8),Error::<Test>::CreditScoreTooLow);
    });
}

#[test]
fn test_delegate_change_validator() {
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4,6,8,10]);
        Credit::initialize_credit(1,95);
        assert_eq!(Credit::get_credit_score(1),Some(95));

        assert_ok!(Delegating::delegate(Origin::signed(1),8));
        let firsrt_ledger = Delegating::credit_ledger(1);
        assert_eq!(firsrt_ledger.validator_account, 8);

        // delegate credit score to a new validator
        assert_ok!(Delegating::delegate(Origin::signed(1),10));
        let second_ledger = Delegating::credit_ledger(1);
        assert_eq!(second_ledger.validator_account, 10);

    });
}

#[test]
fn test_undelegate(){
    new_test_ext().execute_with(|| {
        Delegating::set_current_era(5);
        Delegating::set_candidate_validators(vec![4,6,8,10]);
        Credit::initialize_credit(1,95);
        assert_eq!(Credit::get_credit_score(1),Some(95));

        assert_ok!(Delegating::delegate(Origin::signed(1),8));

        let delegators = Delegating::delegators(5,8);
        println!(" delegators : {:?}", delegators);
        assert_eq!(delegators.contains(&(1,95)), true);

        assert_ok!(Delegating::undelegate(Origin::signed(1)));

        let ledger = Delegating::credit_ledger(1);
        assert_eq!(ledger.withdraw_era, 5 + CREDIT_LOCK_DURATION);

        // remove from validator's delegator list in next era
        Delegating::set_current_era(6);
        let new_delegators = Delegating::delegators(6,8);
        println!(" new delegators : {:?}", new_delegators);
        assert_eq!(new_delegators.contains(&(1,95)), false);
    })
}

#[test]
fn test_undelegate_before_delegate() {
    new_test_ext().execute_with(|| {

        Delegating::set_current_era(5);
        Delegating::set_candidate_validators(vec![4,6,8,10]);
        assert_eq!(Credit::initialize_credit(1,95),true);

        // should be Error with NotDelegate
        assert_noop!(Delegating::undelegate(Origin::signed(1)),Error::<Test>::NotDelegate);

    });
}

#[test]
fn test_withdraw_credit_score(){
    new_test_ext().execute_with(|| {
        Delegating::set_current_era(5);
        Delegating::set_candidate_validators(vec![4,6,8,10]);
        assert_eq!(Credit::initialize_credit(1,95),true);
        assert_ok!(Delegating::delegate(Origin::signed(1),8));
        assert_ok!(Delegating::undelegate(Origin::signed(1)));

        // withdraw before withdraw_era
        assert_noop!(Delegating::withdraw_credit_score(Origin::signed(1)),Error::<Test>::NotRightEra);

        Delegating::set_current_era(90);
        // withdraw with right era
        assert_ok!(Delegating::withdraw_credit_score(Origin::signed(1)));
    });
}

#[test]
fn test_withdraw_without_delegate(){
    new_test_ext().execute_with(|| {

        assert_noop!(Delegating::withdraw_credit_score(Origin::signed(1)),
        Error::<Test>::NoCreditLedgerData);
    });
}

#[test]
fn test_redelegate(){
    new_test_ext().execute_with(|| {
        Delegating::set_current_era(5);
        Delegating::set_candidate_validators(vec![4,6,8,10]);
        assert_eq!(Credit::initialize_credit(1,95),true);
        assert_ok!(Delegating::delegate(Origin::signed(1),8));
        assert_ok!(Delegating::undelegate(Origin::signed(1)));

        assert_ok!(Delegating::redelegate(Origin::signed(1)));
        let ledger = Delegating::credit_ledger(1);
        assert_eq!(ledger.validator_account, 8);

    });
}

#[test]
fn test_set_current_era_validators() {
    new_test_ext().execute_with(|| {
        Delegating::set_current_era_validators(vec![4,6,8,10]);
        assert_eq!(Delegating::current_era_validators(), Some(vec![4,6,8,10]));
    });
}

#[test]
fn test_set_candidates(){
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4,6,8,10]);
        assert_eq!(Delegating::candidate_validators(), Some(vec![4,6,8,10]));
    });
}

#[test]
fn test_get_delegated_score(){
    new_test_ext().execute_with(|| {

        Delegating::set_candidate_validators(vec![4,6,8,10]);
        assert_eq!(Credit::initialize_credit(1,95),true);
        assert_ok!(Delegating::delegate(Origin::signed(1),8));

        assert_eq!(Credit::initialize_credit(2,65),true);
        assert_ok!(Delegating::delegate(Origin::signed(2),8));

        // check delegator's score
        assert_eq!(Delegating::get_delegated_score(1),Some(95));
        assert_eq!(Delegating::get_delegated_score(2),Some(65));

        // check total score
        assert_eq!(Delegating::total_delegated_score(), Some(95 + 65));

        // check total delegated score for validator
        assert_eq!(Delegating::delegated_score_of_validator(&8), Some(95 + 65));

        assert_eq!(Delegating::delegated_score_of_validator(&6), Some(0));

        assert_eq!(Delegating::delegated_score_of_validator(&1), Some(0));

    });
}

#[test]
fn test_set_current_era() {
    new_test_ext().execute_with(|| {

        Delegating::set_current_era(5);

        assert_eq!(Delegating::current_era(), Some(5));

        Delegating::set_current_era(0);
        assert_eq!(Delegating::current_era(), Some(5));

        Delegating::set_current_era(3);
        assert_eq!(Delegating::current_era(), Some(5));

        Delegating::set_current_era(10);
        assert_eq!(Delegating::current_era(), Some(10));
    });
}

#[test]
fn test_kill_credit() {
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4,6,8,10]);

        Delegating::set_current_era(2);

        assert_eq!(Credit::initialize_credit(1,95),true);

        assert_ok!(Delegating::delegate(Origin::signed(1),6));
        let ledger = Delegating::credit_ledger(1);

        // kill a not existed account
        assert_eq!(Delegating::kill_credit(2), false);

        let old_delegators = Delegating::delegators(2,6);
        assert_eq!(old_delegators.contains(&(1,95)), true);

        assert_eq!(Delegating::kill_credit(1),true);

        // delete account from delegator list
        let new_delegators = Delegating::delegators(2,6);
        assert_eq!(new_delegators.contains(&(1,95)), false);
    });
}