use crate::{mock::*, CreditDelegateInterface, Error};
use frame_support::{assert_noop, assert_ok};
use pallet_credit::CreditInterface;

#[test]
fn test_delegate() {
    new_test_ext().execute_with(|| {
        // initialize candidate list
        Delegating::set_candidate_validators(vec![4, 6, 8, 10]);

        // TEST1： delegate to one validator
        // initialize credit score
        let micropayment_vec = vec![(10, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::get_credit_score(10), Some(80));
        // delegate credit score
        assert_ok!(Delegating::delegate(Origin::signed(10), vec![4]));
        // check delegated info
        let info = Delegating::delegated_to_validators(10);
        assert_eq!(info.score, 80);
        assert_eq!(info.validators, vec![4]);
        assert_eq!(Delegating::candidate_delegators(4), vec![(10, 80)]);

        // TEST2： delegate to many validators
        // initialize credit score
        let micropayment_vec = vec![(11, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::get_credit_score(11), Some(80));
        // delegate credit score
        assert_ok!(Delegating::delegate(Origin::signed(11), vec![4, 6, 8, 10]));
        // check delegated info
        let info = Delegating::delegated_to_validators(11);
        assert_eq!(info.score, 80);
        assert_eq!(info.validators, vec![4, 6, 8, 10]);
        assert_eq!(
            Delegating::candidate_delegators(4),
            vec![(10, 80), (11, 20)]
        );
        assert_eq!(Delegating::candidate_delegators(6), vec![(11, 20)]);
        assert_eq!(Delegating::candidate_delegators(8), vec![(11, 20)]);
        assert_eq!(Delegating::candidate_delegators(10), vec![(11, 20)]);

        //  TEST3： delegate with invalid validator
        let micropayment_vec = vec![(19, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_noop!(
            Delegating::delegate(Origin::signed(19), vec![5]),
            Error::<Test>::NotInCandidateValidator
        );

        //  TEST4： delegate with invalid validator
        let micropayment_vec = vec![(20, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_noop!(
            Delegating::delegate(Origin::signed(19), vec![4, 5]),
            Error::<Test>::NotInCandidateValidator
        );

        //  TEST5： delegate with low score
        let micropayment_vec = vec![(21, 60 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_noop!(
            Delegating::delegate(Origin::signed(21), vec![4, 6]),
            Error::<Test>::CreditScoreTooLow
        );

        //  TEST6： delegate after having called delegate()
        let micropayment_vec = vec![(22, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_ok!(Delegating::delegate(Origin::signed(22), vec![4, 6, 8, 10]));
        assert_noop!(
            Delegating::delegate(Origin::signed(22), vec![4]),
            Error::<Test>::AlreadyDelegated
        );
    });
}

#[test]
fn test_undelegate() {
    new_test_ext().execute_with(|| {
        // initialize candidate list
        Delegating::set_candidate_validators(vec![4, 6, 8, 10]);

        // TEST1： undelegate
        // initialize credit score
        let micropayment_vec = vec![(11, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::get_credit_score(11), Some(80));
        // delegate credit score
        assert_ok!(Delegating::delegate(Origin::signed(11), vec![4]));
        // undelegate after calling delegate()
        assert_ok!(Delegating::undelegate(Origin::signed(11)));

        // TEST2: undelegate before calling delegate()
        assert_noop!(
            Delegating::undelegate(Origin::signed(12)),
            Error::<Test>::NotDelegate
        );
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
fn test_set_current_era_validators() {
    new_test_ext().execute_with(|| {
        Delegating::set_current_era_validators(vec![4, 6, 8, 10]);
        assert_eq!(
            Delegating::current_era_validators(),
            Some(vec![4, 6, 8, 10])
        );
    });
}

#[test]
fn test_set_candidates() {
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4, 6, 8, 10]);
        assert_eq!(
            Delegating::get_candidate_validators(),
            Some(vec![4, 6, 8, 10])
        );
    });
}

#[test]
fn test_total_delegated_score() {
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4, 6, 8, 10]);

        let micropayment_vec1 = vec![(1, 90 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec1);
        assert_ok!(Delegating::delegate(Origin::signed(1), vec![4, 6, 8]));

        let micropayment_vec2 = vec![(2, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec2);
        assert_ok!(Delegating::delegate(Origin::signed(2), vec![4, 6, 8, 10]));

        // check total score
        Delegating::set_current_era(4);
        Delegating::set_current_era_validators(vec![4, 6, 8, 10]);
        assert_eq!(Delegating::total_delegated_score(4), Some(90 + 80));
    });
}

#[test]
fn test_get_total_validator_score() {
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4, 6, 8, 10]);
        
        let micropayment_vec1 = vec![(1, 90 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec1);
        assert_ok!(Delegating::delegate(Origin::signed(1), vec![4, 6, 8]));

        let micropayment_vec2 = vec![(2, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec2);
        assert_ok!(Delegating::delegate(Origin::signed(2), vec![4, 6, 8, 10]));

        // check total score
        Delegating::set_current_era(4);
        Delegating::set_current_era_validators(vec![4, 6, 8, 10]);
        assert_eq!(Delegating::total_delegated_score(4), Some(90 + 80));

        Delegating::set_current_era(5);
        Delegating::set_current_era_validators(vec![4, 6, 8]);
        // check total delegated score for validator
        assert_eq!(
            Delegating::get_total_validator_score(Delegating::current_era().unwrap(), 4),
            Some(50)
        );

        assert_eq!(
            Delegating::get_total_validator_score(Delegating::current_era().unwrap(), 6),
            Some(50)
        );

        assert_eq!(
            Delegating::get_total_validator_score(Delegating::current_era().unwrap(), 8),
            Some(50)
        );
    });
}

#[test]
fn test_set_eras_reward() {
    new_test_ext().execute_with(|| {
        Delegating::set_eras_reward(1, 100);
        assert_eq!(Delegating::eras_validator_reward(1), Some(100));
    });
}

#[test]
fn test_poc_slash() {
    new_test_ext().execute_with(|| {
        Delegating::set_candidate_validators(vec![4, 6, 8, 10]);
        let micropayment_vec = vec![(11, 80 * 1_000_000_000_000_000, 5)];
        Credit::update_credit(micropayment_vec);
        assert_eq!(Credit::get_credit_score(11), Some(80));
        assert_ok!(Delegating::delegate(Origin::signed(11), vec![4, 6, 8, 10]));

        Delegating::set_current_era(5);
        Delegating::set_current_era_validators(vec![4, 6, 8]);

        Delegating::poc_slash(&4, 5);
        assert_eq!(Credit::get_credit_score(11), Some(70));
    });
}
