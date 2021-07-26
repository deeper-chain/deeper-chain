// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

//! Tests for the module.

use super::*;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, OnFinalize, OnInitialize, ReservableCurrency},
    StorageMap,
};
use mock::*;
use pallet_balances::Error as BalancesError;
use pallet_credit::CreditInterface;
use sp_runtime::{assert_eq_error_rate, traits::BadOrigin};
use sp_staking::offence::OffenceDetails;
use substrate_test_utils::assert_eq_uvec;

#[test]
fn kill_stash_works() {
    ExtBuilder::default().build_and_execute(|| {
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&11), Some(10));
        // Adds 2 slashing spans
        add_slash(&11);
        // Only can kill a stash account
        assert_noop!(Staking::kill_stash(&12, 0), Error::<Test>::NotStash);
        // Respects slashing span count
        assert_noop!(
            Staking::kill_stash(&11, 0),
            Error::<Test>::IncorrectSlashingSpans
        );
        // Correct inputs, everything works
        assert_ok!(Staking::kill_stash(&11, 2));
        // No longer bonded.
        assert_eq!(Staking::bonded(&11), None);
    });
}

#[test]
fn basic_setup_works() {
    // Verifies initial conditions of mock
    ExtBuilder::default().build_and_execute(|| {
        // Account 11 is stashed and locked, and account 10 is the controller
        assert_eq!(Staking::bonded(&11), Some(10));
        // Account 21 is stashed and locked, and account 20 is the controller
        assert_eq!(Staking::bonded(&21), Some(20));
        // Account 1 is not a stashed
        assert_eq!(Staking::bonded(&1), None);

        // Account 10 controls the stash from account 11, which is 100 * balance_factor units
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![]
            })
        );
        // Account 20 controls the stash from account 21, which is 200 * balance_factor units
        assert_eq!(
            Staking::ledger(&20),
            Some(StakingLedger {
                stash: 21,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![]
            })
        );
        // Account 1 does not control any stash
        assert_eq!(Staking::ledger(&1), None);

        // ValidatorPrefs are default
        assert_eq_uvec!(
            <Validators<Test>>::iter().collect::<Vec<_>>(),
            vec![
                (31, ValidatorPrefs::default()),
                (21, ValidatorPrefs::default()),
                (11, ValidatorPrefs::default())
            ]
        );

        assert_eq!(
            Staking::ledger(100),
            Some(StakingLedger {
                stash: 101,
                total: 500,
                active: 500,
                unlocking: vec![],
                claimed_rewards: vec![]
            })
        );
        /* TODO fix
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1125,
                own: 1000,
                others: vec![IndividualExposure {
                    who: 101,
                    value: 125
                }]
            },
        );
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
            Exposure {
                total: 1375,
                own: 1000,
                others: vec![IndividualExposure {
                    who: 101,
                    value: 375
                }]
            },
        );

        // initial total stake = 1125 + 1375
        assert_eq!(
            Staking::eras_total_stake(Staking::active_era().unwrap().index),
            2500
        ); */

        // The number of validators required.
        assert_eq!(Staking::validator_count(), 2);

        // Initial Era and session
        assert_eq!(Staking::active_era().unwrap().index, 0);

        // Account 10 has `balance_factor` free balance
        assert_eq!(Balances::free_balance(10), 1);
        assert_eq!(Balances::free_balance(10), 1);

        // New era is not being forced
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
    });
}

#[test]
fn change_controller_works() {
    ExtBuilder::default().build_and_execute(|| {
        // 10 and 11 are bonded as stash controller.
        assert_eq!(Staking::bonded(&11), Some(10));

        // 10 can control 11 who is initially a validator.
        assert_ok!(Staking::chill(Origin::signed(10)));

        // change controller
        assert_ok!(Staking::set_controller(Origin::signed(11), 5));
        assert_eq!(Staking::bonded(&11), Some(5));
        mock::start_active_era(1);

        // 10 is no longer in control.
        assert_noop!(
            Staking::validate(Origin::signed(10), ValidatorPrefs::default()),
            Error::<Test>::NotController,
        );
        assert_ok!(Staking::validate(
            Origin::signed(5),
            ValidatorPrefs::default()
        ));
    })
}

#[test]
fn rewards_should_work() {
    ExtBuilder::default()
        .nominate(true)
        .session_per_era(3)
        .build_and_execute(|| {
            let init_balance_10 = Balances::total_balance(&10);
            let init_balance_11 = Balances::total_balance(&11);
            let init_balance_20 = Balances::total_balance(&20);
            let init_balance_21 = Balances::total_balance(&21);
            let init_balance_100 = Balances::total_balance(&100);
            let init_balance_101 = Balances::total_balance(&101);

            // Set payees
            Payee::<Test>::insert(11, RewardDestination::Controller);
            Payee::<Test>::insert(21, RewardDestination::Controller);
            Payee::<Test>::insert(101, RewardDestination::Controller);

            // TODO implement the test
        });
}

#[test]
fn less_than_needed_candidates_works() {
    ExtBuilder::default()
        .minimum_validator_count(1)
        .validator_count(4)
        .nominate(false)
        .num_validators(3)
        .build()
        .execute_with(|| {
            assert_eq!(Staking::validator_count(), 4);
            assert_eq!(Staking::minimum_validator_count(), 1);
            assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

            mock::start_active_era(1);

            // Previous set is selected. NO election algorithm is even executed.
            assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

            // But the exposure is updated in a simple way. No external votes exists.
            // This is purely self-vote.
            assert!(
                ErasStakers::<Test>::iter_prefix_values(Staking::active_era().unwrap().index)
                    .all(|exposure| exposure.others.is_empty())
            );
        });
}

#[test]
fn no_candidate_emergency_condition() {
    ExtBuilder::default()
        .minimum_validator_count(1)
        .validator_count(15)
        .num_validators(4)
        .validator_pool(true)
        .nominate(false)
        .build()
        .execute_with(|| {
            // initial validators
            assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
            let prefs = ValidatorPrefs {
                commission: Perbill::one(),
                ..Default::default()
            };
            <Staking as crate::Store>::Validators::insert(11, prefs.clone());

            // set the minimum validator count.
            <Staking as crate::Store>::MinimumValidatorCount::put(10);

            // try to chill
            let _ = Staking::chill(Origin::signed(10));

            // trigger era
            mock::start_active_era(1);

            // Previous ones are elected. chill is invalidates. TODO: #2494
            assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
            // Though the validator preferences has been removed.
            assert!(Staking::validators(11) != prefs);
        });
}

// TODO
/*
#[test]
fn nominators_also_get_slashed_pro_rata() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);
        let slash_percent = Perbill::from_percent(5);
        let initial_exposure = Staking::eras_stakers(active_era(), 11);
        // 101 is a nominator for 11
        assert_eq!(initial_exposure.others.first().unwrap().who, 101,);

        // staked values;
        let nominator_stake = Staking::ledger(100).unwrap().active;
        let nominator_balance = balances(&101).0;
        let validator_stake = Staking::ledger(10).unwrap().active;
        let validator_balance = balances(&11).0;
        let exposed_stake = initial_exposure.total;
        let exposed_validator = initial_exposure.own;
        let exposed_nominator = initial_exposure.others.first().unwrap().value;

        // 11 goes offline
        on_offence_now(
            &[OffenceDetails {
                offender: (11, initial_exposure.clone()),
                reporters: vec![],
            }],
            &[slash_percent],
        );

        // both stakes must have been decreased.
        assert!(Staking::ledger(100).unwrap().active < nominator_stake);
        assert!(Staking::ledger(10).unwrap().active < validator_stake);

        let slash_amount = slash_percent * exposed_stake;
        let validator_share =
            Perbill::from_rational_approximation(exposed_validator, exposed_stake) * slash_amount;
        let nominator_share =
            Perbill::from_rational_approximation(exposed_nominator, exposed_stake) * slash_amount;

        // both slash amounts need to be positive for the test to make sense.
        assert!(validator_share > 0);
        assert!(nominator_share > 0);

        // both stakes must have been decreased pro-rata.
        assert_eq!(
            Staking::ledger(100).unwrap().active,
            nominator_stake - nominator_share,
        );
        assert_eq!(
            Staking::ledger(10).unwrap().active,
            validator_stake - validator_share,
        );
        assert_eq!(
            balances(&101).0, // free balance
            nominator_balance - nominator_share,
        );
        assert_eq!(
            balances(&11).0, // free balance
            validator_balance - validator_share,
        );
        // Because slashing happened.
        assert!(is_disabled(10));
    });
}*/

#[test]
fn double_controlling_should_fail() {
    // should test (in the same order):
    // * an account already bonded as controller CANNOT be reused as the controller of another account.
    ExtBuilder::default().build_and_execute(|| {
        let arbitrary_value = 5;
        // 2 = controller, 1 stashed => ok
        assert_ok!(Staking::bond(
            Origin::signed(1),
            2,
            arbitrary_value,
            RewardDestination::default(),
        ));
        // 2 = controller, 3 stashed (Note that 2 is reused.) => no-op
        assert_noop!(
            Staking::bond(
                Origin::signed(3),
                2,
                arbitrary_value,
                RewardDestination::default()
            ),
            Error::<Test>::AlreadyPaired,
        );
    });
}

#[test]
fn session_and_eras_work_simple() {
    ExtBuilder::default().period(1).build_and_execute(|| {
        assert_eq!(active_era(), 0);
        assert_eq!(current_era(), 0);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(System::block_number(), 1);

        // Session 1: this is basically a noop. This has already been started.
        start_session(1);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(active_era(), 0);
        assert_eq!(System::block_number(), 1);

        // Session 2: No change.
        start_session(2);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(active_era(), 0);
        assert_eq!(System::block_number(), 2);

        // Session 3: Era increment.
        start_session(3);
        assert_eq!(Session::current_index(), 3);
        assert_eq!(active_era(), 1);
        assert_eq!(System::block_number(), 3);

        // Session 4: No change.
        start_session(4);
        assert_eq!(Session::current_index(), 4);
        assert_eq!(active_era(), 1);
        assert_eq!(System::block_number(), 4);

        // Session 5: No change.
        start_session(5);
        assert_eq!(Session::current_index(), 5);
        assert_eq!(active_era(), 1);
        assert_eq!(System::block_number(), 5);

        // Session 6: Era increment.
        start_session(6);
        assert_eq!(Session::current_index(), 6);
        assert_eq!(active_era(), 2);
        assert_eq!(System::block_number(), 6);
    });
}

#[test]
fn session_and_eras_work_complex() {
    ExtBuilder::default().period(5).build_and_execute(|| {
        assert_eq!(active_era(), 0);
        assert_eq!(Session::current_index(), 0);
        assert_eq!(System::block_number(), 1);

        start_session(1);
        assert_eq!(Session::current_index(), 1);
        assert_eq!(active_era(), 0);
        assert_eq!(System::block_number(), 5);

        start_session(2);
        assert_eq!(Session::current_index(), 2);
        assert_eq!(active_era(), 0);
        assert_eq!(System::block_number(), 10);

        start_session(3);
        assert_eq!(Session::current_index(), 3);
        assert_eq!(active_era(), 1);
        assert_eq!(System::block_number(), 15);

        start_session(4);
        assert_eq!(Session::current_index(), 4);
        assert_eq!(active_era(), 1);
        assert_eq!(System::block_number(), 20);

        start_session(5);
        assert_eq!(Session::current_index(), 5);
        assert_eq!(active_era(), 1);
        assert_eq!(System::block_number(), 25);

        start_session(6);
        assert_eq!(Session::current_index(), 6);
        assert_eq!(active_era(), 2);
        assert_eq!(System::block_number(), 30);
    });
}

#[test]
fn forcing_new_era_works() {
    ExtBuilder::default().build_and_execute(|| {
        // normal flow of session.
        start_session(1);
        assert_eq!(active_era(), 0);

        start_session(2);
        assert_eq!(active_era(), 0);

        start_session(3);
        assert_eq!(active_era(), 1);

        // no era change.
        ForceEra::put(Forcing::ForceNone);

        start_session(4);
        assert_eq!(active_era(), 1);

        start_session(5);
        assert_eq!(active_era(), 1);

        start_session(6);
        assert_eq!(active_era(), 1);

        start_session(7);
        assert_eq!(active_era(), 1);

        // back to normal.
        // this immediately starts a new session.
        ForceEra::put(Forcing::NotForcing);

        start_session(8);
        assert_eq!(active_era(), 1);

        start_session(9);
        assert_eq!(active_era(), 2);
        // forceful change
        ForceEra::put(Forcing::ForceAlways);

        start_session(10);
        assert_eq!(active_era(), 2);

        start_session(11);
        assert_eq!(active_era(), 3);

        start_session(12);
        assert_eq!(active_era(), 4);

        // just one forceful change
        ForceEra::put(Forcing::ForceNew);
        start_session(13);
        assert_eq!(active_era(), 5);
        assert_eq!(ForceEra::get(), Forcing::NotForcing);

        start_session(14);
        assert_eq!(active_era(), 6);

        start_session(15);
        assert_eq!(active_era(), 6);
    });
}

#[test]
fn cannot_transfer_staked_balance() {
    // Tests that a stash account cannot transfer funds
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // Confirm account 11 is stashed
        assert_eq!(Staking::bonded(&11), Some(10)); // TODO fix bonded logic
                                                    // Confirm account 11 has some free balance
        assert_eq!(Balances::free_balance(11), 1000);
        // Confirm account 11 (via controller 10) is totally staked
        //assert_eq!(Staking::eras_stakers(active_era(), 11).total, 1000);
        // Confirm account 11 cannot transfer as a result
        assert_noop!(
            Balances::transfer(Origin::signed(11), 20, 1),
            BalancesError::<Test, _>::LiquidityRestrictions
        );

        // Give account 11 extra free balance
        let _ = Balances::make_free_balance_be(&11, 10000);
        // Confirm that account 11 can now transfer some balance
        assert_ok!(Balances::transfer(Origin::signed(11), 20, 1));
    });
}

#[test]
fn cannot_transfer_staked_balance_2() {
    // Tests that a stash account cannot transfer funds
    // Same test as above but with 20, and more accurate.
    // 21 has 2000 free balance but 1000 at stake
    ExtBuilder::default()
        .nominate(false)
        .fair(true)
        .build_and_execute(|| {
            // Confirm account 21 is stashed
            assert_eq!(Staking::bonded(&21), Some(20)); // TODO fix bonded logic
                                                        // Confirm account 21 has some free balance
            assert_eq!(Balances::free_balance(21), 2000);
            // Confirm account 21 (via controller 20) is totally staked
            //assert_eq!(
            //    Staking::eras_stakers(Staking::active_era().unwrap().index, 21).total,
            //    1000
            //);
            // Confirm account 21 can transfer at most 1000
            assert_noop!(
                Balances::transfer(Origin::signed(21), 20, 1001),
                BalancesError::<Test, _>::LiquidityRestrictions
            );
            assert_ok!(Balances::transfer(Origin::signed(21), 20, 1000));
        });
}

#[test]
fn cannot_reserve_staked_balance() {
    // Checks that a bonded account cannot reserve balance from free balance
    ExtBuilder::default().build_and_execute(|| {
        // Confirm account 11 is stashed
        assert_eq!(Staking::bonded(&11), Some(10)); // TODO fix bonded logic
                                                    // Confirm account 11 has some free balance
        assert_eq!(Balances::free_balance(11), 1000);
        // Confirm account 11 (via controller 10) is totally staked
        /*
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).own,
            1000
        );*/
        // Confirm account 11 cannot reserve as a result
        assert_noop!(
            Balances::reserve(&11, 1),
            BalancesError::<Test, _>::LiquidityRestrictions,
        );

        // Give account 11 extra free balance
        let _ = Balances::make_free_balance_be(&11, 10000);
        // Confirm account 11 can now reserve balance
        assert_ok!(Balances::reserve(&11, 1));
    });
}

#[test]
fn bond_extra_works() {
    // Tests that extra `free_balance` in the stash can be added to stake
    // NOTE: this tests only verifies `StakingLedger` for correct updates
    // See `bond_extra_and_withdraw_unbonded_works` for more details and updates on `Exposure`.
    ExtBuilder::default().build_and_execute(|| {
        // Check that account 10 is a validator
        assert!(<Validators<Test>>::contains_key(11));
        // Check that account 10 is bonded to account 11
        assert_eq!(Staking::bonded(&11), Some(10));
        // Check how much is at stake
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );

        // Give account 11 some large free balance greater than total
        let _ = Balances::make_free_balance_be(&11, 1000000);

        // Call the bond_extra function from controller, add only 100
        assert_ok!(Staking::bond_extra(Origin::signed(11), 100));
        // There should be 100 more `total` and `active` in the ledger
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );

        // Call the bond_extra function with a large number, should handle it
        assert_ok!(Staking::bond_extra(
            Origin::signed(11),
            Balance::max_value()
        ));
        // The full amount of the funds should now be in the total and active
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000000,
                active: 1000000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
    });
}

#[test]
fn bond_extra_and_withdraw_unbonded_works() {
    // * Should test
    // * Given an account being bonded [and chosen as a validator](not mandatory)
    // * It can add extra funds to the bonded account.
    // * it can unbond a portion of its funds from the stash account.
    // * Once the unbonding period is done, it can actually take the funds out of the stash.
    ExtBuilder::default().nominate(false).build_and_execute(|| {
        // Set payee to controller. avoids confusion
        assert_ok!(Staking::set_payee(
            Origin::signed(10),
            RewardDestination::Controller
        ));

        // Give account 11 some large free balance greater than total
        let _ = Balances::make_free_balance_be(&11, 1000000);

        // Initial config should be correct
        assert_eq!(Staking::active_era().unwrap().index, 0);

        // check the balance of a validator accounts.
        assert_eq!(Balances::total_balance(&10), 1);

        // confirm that 10 is a normal validator and gets paid at the end of the era.
        mock::start_active_era(1);

        // Initial state of 10
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000,
                active: 1000,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        /*
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 500,
                own: 500,
                others: vec![]
            }
        );*/

        // deposit the extra 100 units
        Staking::bond_extra(Origin::signed(11), 100).unwrap();

        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        // Exposure is a snapshot! only updated after the next era update.
        /*
        assert_ne!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1000 + 100,
                own: 1000 + 100,
                others: vec![]
            }
        );*/

        // trigger next era.
        mock::start_active_era(2);
        assert_eq!(Staking::active_era().unwrap().index, 2);

        // ledger should be the same.
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 1000 + 100,
                unlocking: vec![],
                claimed_rewards: vec![],
            })
        );
        // Exposure is now updated.
        /*
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
            Exposure {
                total: 1000 + 100,
                own: 1000 + 100,
                others: vec![]
            }
        );*/

        // Unbond almost all of the funds in stash.
        Staking::unbond(Origin::signed(10), 1000).unwrap();
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 100,
                unlocking: vec![UnlockChunk {
                    value: 1000,
                    era: 2 + 3
                }],
                claimed_rewards: vec![]
            }),
        );

        // Attempting to free the balances now will fail. 2 eras need to pass.
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 100,
                unlocking: vec![UnlockChunk {
                    value: 1000,
                    era: 2 + 3
                }],
                claimed_rewards: vec![]
            }),
        );

        // trigger next era.
        mock::start_active_era(3);

        // nothing yet
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 1000 + 100,
                active: 100,
                unlocking: vec![UnlockChunk {
                    value: 1000,
                    era: 2 + 3
                }],
                claimed_rewards: vec![]
            }),
        );

        // trigger next era.
        mock::start_active_era(5);

        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));
        // Now the value is free and the staking ledger is updated.
        assert_eq!(
            Staking::ledger(&10),
            Some(StakingLedger {
                stash: 11,
                total: 100,
                active: 100,
                unlocking: vec![],
                claimed_rewards: vec![]
            }),
        );
    })
}

#[test]
fn too_many_unbond_calls_should_not_work() {
    ExtBuilder::default().build_and_execute(|| {
        // locked at era 0 until 3
        for _ in 0..MAX_UNLOCKING_CHUNKS - 1 {
            assert_ok!(Staking::unbond(Origin::signed(10), 1));
        }

        mock::start_active_era(1);

        // locked at era 1 until 4
        assert_ok!(Staking::unbond(Origin::signed(10), 1));
        // can't do more.
        assert_noop!(
            Staking::unbond(Origin::signed(10), 1),
            Error::<Test>::NoMoreChunks
        );

        mock::start_active_era(3);

        assert_noop!(
            Staking::unbond(Origin::signed(10), 1),
            Error::<Test>::NoMoreChunks
        );
        // free up.
        assert_ok!(Staking::withdraw_unbonded(Origin::signed(10), 0));

        // Can add again.
        assert_ok!(Staking::unbond(Origin::signed(10), 1));
        assert_eq!(Staking::ledger(&10).unwrap().unlocking.len(), 2);
    })
}

#[test]
fn rebond_works() {
    // * Should test
    // * Given an account being bonded [and chosen as a validator](not mandatory)
    // * it can unbond a portion of its funds from the stash account.
    // * it can re-bond a portion of the funds scheduled to unlock.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Set payee to controller. avoids confusion
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Controller
            ));

            // Give account 11 some large free balance greater than total
            let _ = Balances::make_free_balance_be(&11, 1000000);

            // confirm that 10 is a normal validator and gets paid at the end of the era.
            mock::start_active_era(1);

            // Initial state of 10
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(2);
            assert_eq!(Staking::active_era().unwrap().index, 2);

            // Try to rebond some funds. We get an error since no fund is unbonded.
            assert_noop!(
                Staking::rebond(Origin::signed(10), 500),
                Error::<Test>::NoUnlockChunk,
            );

            // Unbond almost all of the funds in stash.
            Staking::unbond(Origin::signed(10), 900).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![UnlockChunk {
                        value: 900,
                        era: 2 + 3,
                    }],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond all the funds unbonded.
            Staking::rebond(Origin::signed(10), 900).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            // Unbond almost all of the funds in stash.
            Staking::unbond(Origin::signed(10), 900).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![UnlockChunk { value: 900, era: 5 }],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond part of the funds unbonded.
            Staking::rebond(Origin::signed(10), 500).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 600,
                    unlocking: vec![UnlockChunk { value: 400, era: 5 }],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond the remainder of the funds unbonded.
            Staking::rebond(Origin::signed(10), 500).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            // Unbond parts of the funds in stash.
            Staking::unbond(Origin::signed(10), 300).unwrap();
            Staking::unbond(Origin::signed(10), 300).unwrap();
            Staking::unbond(Origin::signed(10), 300).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![
                        UnlockChunk { value: 300, era: 5 },
                        UnlockChunk { value: 300, era: 5 },
                        UnlockChunk { value: 300, era: 5 },
                    ],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond part of the funds unbonded.
            Staking::rebond(Origin::signed(10), 500).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 600,
                    unlocking: vec![
                        UnlockChunk { value: 300, era: 5 },
                        UnlockChunk { value: 100, era: 5 },
                    ],
                    claimed_rewards: vec![],
                })
            );
        })
}

#[test]
fn rebond_is_fifo() {
    // Rebond should proceed by reversing the most recent bond operations.
    ExtBuilder::default()
        .nominate(false)
        .build()
        .execute_with(|| {
            // Set payee to controller. avoids confusion
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Controller
            ));

            // Give account 11 some large free balance greater than total
            let _ = Balances::make_free_balance_be(&11, 1000000);

            // confirm that 10 is a normal validator and gets paid at the end of the era.
            mock::start_active_era(1);

            // Initial state of 10
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(2);

            // Unbond some of the funds in stash.
            Staking::unbond(Origin::signed(10), 400).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 600,
                    unlocking: vec![UnlockChunk {
                        value: 400,
                        era: 2 + 3
                    },],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(3);

            // Unbond more of the funds in stash.
            Staking::unbond(Origin::signed(10), 300).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 300,
                    unlocking: vec![
                        UnlockChunk {
                            value: 400,
                            era: 2 + 3
                        },
                        UnlockChunk {
                            value: 300,
                            era: 3 + 3
                        },
                    ],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(4);

            // Unbond yet more of the funds in stash.
            Staking::unbond(Origin::signed(10), 200).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 100,
                    unlocking: vec![
                        UnlockChunk {
                            value: 400,
                            era: 2 + 3
                        },
                        UnlockChunk {
                            value: 300,
                            era: 3 + 3
                        },
                        UnlockChunk {
                            value: 200,
                            era: 4 + 3
                        },
                    ],
                    claimed_rewards: vec![],
                })
            );

            // Re-bond half of the unbonding funds.
            Staking::rebond(Origin::signed(10), 400).unwrap();
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 500,
                    unlocking: vec![
                        UnlockChunk {
                            value: 400,
                            era: 2 + 3
                        },
                        UnlockChunk {
                            value: 100,
                            era: 3 + 3
                        },
                    ],
                    claimed_rewards: vec![],
                })
            );
        })
}

#[test]
fn on_free_balance_zero_stash_removes_validator() {
    // Tests that validator storage items are cleaned up when stash is empty
    // Tests that storage items are untouched when controller is empty
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // Check the balance of the validator account
            assert_eq!(Balances::free_balance(10), 256);
            // Check the balance of the stash account
            assert_eq!(Balances::free_balance(11), 256000);
            // Check these two accounts are bonded
            assert_eq!(Staking::bonded(&11), Some(10));

            // Set some storage items which we expect to be cleaned up
            // Set payee information
            assert_ok!(Staking::set_payee(
                Origin::signed(10),
                RewardDestination::Stash
            ));

            // Check storage items that should be cleaned up
            assert!(<Ledger<Test>>::contains_key(&10));
            assert!(<Bonded<Test>>::contains_key(&11));
            assert!(<Validators<Test>>::contains_key(&11));
            assert!(<Payee<Test>>::contains_key(&11));

            // Reduce free_balance of controller to 0
            let _ = Balances::slash(&10, Balance::max_value());

            // Check the balance of the stash account has not been touched
            assert_eq!(Balances::free_balance(11), 256000);
            // Check these two accounts are still bonded
            assert_eq!(Staking::bonded(&11), Some(10));

            // Check storage items have not changed
            assert!(<Ledger<Test>>::contains_key(&10));
            assert!(<Bonded<Test>>::contains_key(&11));
            assert!(<Validators<Test>>::contains_key(&11));
            assert!(<Payee<Test>>::contains_key(&11));

            // Reduce free_balance of stash to 0
            let _ = Balances::slash(&11, Balance::max_value());
            // Check total balance of stash
            assert_eq!(Balances::total_balance(&11), 10);

            // Reap the stash
            assert_ok!(Staking::reap_stash(Origin::none(), 11, 0));

            // Check storage items do not exist
            assert!(!<Ledger<Test>>::contains_key(&10));
            assert!(!<Bonded<Test>>::contains_key(&11));
            assert!(!<Validators<Test>>::contains_key(&11));
            assert!(!<Payee<Test>>::contains_key(&11));
        });
}

#[test]
fn on_low_credit_score_removes_delegator() {
    // Tests that nominator storage items are cleaned up when stash is empty
    // Tests that storage items are untouched when controller is empty
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            //TODO
        });
}

#[test]
fn election_works() {
    ExtBuilder::default()
        .nominate(false)
        .validator_pool(true)
        .build_and_execute(|| {
            assert_eq_uvec!(validator_controllers(), vec![10, 40]);

            // add 1 delegator
            assert_ok!(Staking::delegate(Origin::signed(1), vec![31, 41]));
            println!(
                "31 CandidateDelegators: {:?}",
                Staking::candidate_delegators(31)
            );
            println!(
                "41 CandidateDelegators: {:?}",
                Staking::candidate_delegators(41)
            );

            // new block
            mock::start_active_era(1);
            assert_eq_uvec!(validator_controllers(), vec![40, 30]);
        });
}

#[test]
fn bond_with_no_staked_value() {
    // Behavior when someone bonds with no staked value.
    // Particularly when she votes and the candidate is elected.
    ExtBuilder::default()
        .validator_count(3)
        .existential_deposit(5)
        .nominate(false)
        .minimum_validator_count(1)
        .build()
        .execute_with(|| {
            // Can't bond with 1
            assert_noop!(
                Staking::bond(Origin::signed(1), 2, 1, RewardDestination::Controller),
                Error::<Test>::InsufficientValue,
            );
            // bonded with absolute minimum value possible.
            assert_ok!(Staking::bond(
                Origin::signed(1),
                2,
                5,
                RewardDestination::Controller
            ));
            assert_eq!(Balances::locks(&1)[0].amount, 5);

            // unbonding even 1 will cause all to be unbonded.
            assert_ok!(Staking::unbond(Origin::signed(2), 1));
            assert_eq!(
                Staking::ledger(2),
                Some(StakingLedger {
                    stash: 1,
                    active: 0,
                    total: 5,
                    unlocking: vec![UnlockChunk { value: 5, era: 3 }],
                    claimed_rewards: vec![],
                })
            );

            mock::start_active_era(1);
            mock::start_active_era(2);

            // not yet removed.
            assert_ok!(Staking::withdraw_unbonded(Origin::signed(2), 0));
            assert!(Staking::ledger(2).is_some());
            assert_eq!(Balances::locks(&1)[0].amount, 5);

            mock::start_active_era(3);

            // poof. Account 1 is removed from the staking system.
            assert_ok!(Staking::withdraw_unbonded(Origin::signed(2), 0));
            assert!(Staking::ledger(2).is_none());
            assert_eq!(Balances::locks(&1).len(), 0);
        });
}

#[test]
fn new_era_elects_correct_number_of_validators() {
    ExtBuilder::default()
        .nominate(true)
        .validator_pool(true)
        .fair(true)
        .validator_count(1)
        .build()
        .execute_with(|| {
            assert_eq!(Staking::validator_count(), 1);
            assert_eq!(validator_controllers().len(), 1);

            Session::on_initialize(System::block_number());

            assert_eq!(validator_controllers().len(), 1);
        })
}

#[test]
fn reward_from_authorship_event_handler_works() {
    ExtBuilder::default().build_and_execute(|| {
        use pallet_authorship::EventHandler;

        assert_eq!(<pallet_authorship::Module<Test>>::author(), 11);

        <Module<Test>>::note_author(11);
        <Module<Test>>::note_uncle(21, 1);
        // Rewarding the same two times works.
        <Module<Test>>::note_uncle(11, 1);

        // Not mandatory but must be coherent with rewards
        assert_eq_uvec!(Session::validators(), vec![11, 21]);

        // 21 is rewarded as an uncle producer
        // 11 is rewarded as a block producer and uncle referencer and uncle producer
        assert_eq!(
            ErasRewardPoints::<Test>::get(Staking::active_era().unwrap().index),
            EraRewardPoints {
                individual: vec![(11, 20 + 2 * 2 + 1), (21, 1)].into_iter().collect(),
                total: 26,
            },
        );
    })
}

#[test]
fn add_reward_points_fns_works() {
    ExtBuilder::default().build_and_execute(|| {
        // Not mandatory but must be coherent with rewards
        assert_eq_uvec!(Session::validators(), vec![21, 11]);

        <Module<Test>>::reward_by_ids(vec![(21, 1), (11, 1), (11, 1)]);

        <Module<Test>>::reward_by_ids(vec![(21, 1), (11, 1), (11, 1)]);

        assert_eq!(
            ErasRewardPoints::<Test>::get(Staking::active_era().unwrap().index),
            EraRewardPoints {
                individual: vec![(11, 4), (21, 2)].into_iter().collect(),
                total: 6,
            },
        );
    })
}

#[test]
fn unbonded_balance_is_not_slashable() {
    ExtBuilder::default().build_and_execute(|| {
        // total amount staked is slashable.
        assert_eq!(Staking::slashable_balance_of(&11), 1000);

        assert_ok!(Staking::unbond(Origin::signed(10), 800));

        // only the active portion.
        assert_eq!(Staking::slashable_balance_of(&11), 200);
    })
}

#[test]
fn era_is_always_same_length() {
    // This ensures that the sessions is always of the same length if there is no forcing no
    // session changes.
    ExtBuilder::default().build_and_execute(|| {
        let session_per_era = <SessionsPerEra as Get<SessionIndex>>::get();

        mock::start_active_era(1);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session_per_era
        );

        mock::start_active_era(2);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session_per_era * 2u32
        );

        let session = Session::current_index();
        ForceEra::put(Forcing::ForceNew);
        advance_session();
        advance_session();
        assert_eq!(current_era(), 3);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session + 2
        );

        mock::start_active_era(4);
        assert_eq!(
            Staking::eras_start_session_index(current_era()).unwrap(),
            session + 2u32 + session_per_era
        );
    });
}

#[test]
fn offence_forces_new_era() {
    ExtBuilder::default().build_and_execute(|| {
        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(5)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
    });
}

#[test]
fn offence_ensures_new_era_without_clobbering() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(Staking::force_new_era_always(Origin::root()));
        assert_eq!(Staking::force_era(), Forcing::ForceAlways);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(5)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceAlways);
    });
}

#[test]
fn offence_deselects_validator_even_when_slash_is_zero() {
    ExtBuilder::default().build_and_execute(|| {
        assert!(Session::validators().contains(&11));
        assert!(<Validators<Test>>::contains_key(11));

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
        assert!(!<Validators<Test>>::contains_key(11));

        mock::start_active_era(1);

        assert!(!Session::validators().contains(&11));
        assert!(!<Validators<Test>>::contains_key(11));
    });
}

#[test]
fn slashing_performed_according_exposure() {
    // This test checks that slashing is performed according the exposure (or more precisely,
    // historical exposure), not the current balance.
    ExtBuilder::default().build_and_execute(|| {
        // TODO
        /*
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).own,
            1000
        );*/

        // Handle an offence with a historical exposure.
        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Exposure {
                        total: 500,
                        own: 500,
                        others: vec![],
                    },
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(50)],
        );

        // The stash account should be slashed for 250 (50% of 500).
        assert_eq!(Balances::free_balance(11), 1000 - 250);
    });
}

//TODO clarify the requirements, and update test
/*
#[test]
fn slash_in_old_span_does_not_deselect() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);

        assert!(<Validators<Test>>::contains_key(11));
        assert!(Session::validators().contains(&11));

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        assert_eq!(Staking::force_era(), Forcing::ForceNew);
        assert!(!<Validators<Test>>::contains_key(11));

        mock::start_active_era(2);

        Staking::validate(Origin::signed(10), Default::default()).unwrap();
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        assert!(<Validators<Test>>::contains_key(11));
        assert!(!Session::validators().contains(&11));

        mock::start_active_era(3);

        // this staker is in a new slashing span now, having re-registered after
        // their prior slash.

        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
            1,
        );

        // not forcing for zero-slash and previous span.
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        assert!(<Validators<Test>>::contains_key(11));
        assert!(Session::validators().contains(&11));

        on_offence_in_era(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            // NOTE: A 100% slash here would clean up the account, causing de-registration.
            &[Perbill::from_percent(95)],
            1,
        );

        // or non-zero.
        assert_eq!(Staking::force_era(), Forcing::NotForcing);
        assert!(<Validators<Test>>::contains_key(11));
        assert!(Session::validators().contains(&11));
    });
}
*/

// TODO uncomment it after fixing the bond function
/*
#[test]
fn reporters_receive_their_slice() {
    // This test verifies that the reporters of the offence receive their slice from the slashed
    // amount.
    ExtBuilder::default().build_and_execute(|| {
        // The reporters' reward is calculated from the total exposure.
        let initial_balance = 1125;

        /*
        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).total,
            initial_balance
        );*/

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![1, 2],
            }],
            &[Perbill::from_percent(50)],
        );

        // F1 * (reward_proportion * slash - 0)
        // 50% * (10% * initial_balance / 2)
        let reward = (initial_balance / 20) / 2;
        let reward_each = reward / 2; // split into two pieces.
        assert_eq!(Balances::free_balance(1), 10 + reward_each);
        assert_eq!(Balances::free_balance(2), 20 + reward_each);
    });
}

#[test]
fn subsequent_reports_in_same_span_pay_out_less() {
    // This test verifies that the reporters of the offence receive their slice from the slashed
    // amount, but less and less if they submit multiple reports in one span.
    ExtBuilder::default().build_and_execute(|| {
        // The reporters' reward is calculated from the total exposure.
        let initial_balance = 1125;

        assert_eq!(
            Staking::eras_stakers(Staking::active_era().unwrap().index, 11).total,
            initial_balance
        );

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![1],
            }],
            &[Perbill::from_percent(20)],
        );

        // F1 * (reward_proportion * slash - 0)
        // 50% * (10% * initial_balance * 20%)
        let reward = (initial_balance / 5) / 20;
        assert_eq!(Balances::free_balance(1), 10 + reward);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![1],
            }],
            &[Perbill::from_percent(50)],
        );

        let prior_payout = reward;

        // F1 * (reward_proportion * slash - prior_payout)
        // 50% * (10% * (initial_balance / 2) - prior_payout)
        let reward = ((initial_balance / 20) - prior_payout) / 2;
        assert_eq!(Balances::free_balance(1), 10 + prior_payout + reward);
    });
}

#[test]
#[ignore = "We will update Proof of Credit"]
fn invulnerables_are_not_slashed() {
    // For invulnerable validators no slashing is performed.
    ExtBuilder::default()
        .invulnerables(vec![11])
        .build_and_execute(|| {
            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(21), 2000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 21);
            let initial_balance = Staking::slashable_balance_of(&21);

            let nominator_balances: Vec<_> = exposure
                .others
                .iter()
                .map(|o| Balances::free_balance(&o.who))
                .collect();

            on_offence_now(
                &[
                    OffenceDetails {
                        offender: (
                            11,
                            Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                        ),
                        reporters: vec![],
                    },
                    OffenceDetails {
                        offender: (
                            21,
                            Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                        ),
                        reporters: vec![],
                    },
                ],
                &[Perbill::from_percent(50), Perbill::from_percent(20)],
            );

            // The validator 11 hasn't been slashed, but 21 has been.
            assert_eq!(Balances::free_balance(11), 1000);
            // 2000 - (0.2 * initial_balance)
            assert_eq!(
                Balances::free_balance(21),
                2000 - (2 * initial_balance / 10)
            );

            // ensure that nominators were slashed as well.
            for (initial_balance, other) in nominator_balances.into_iter().zip(exposure.others) {
                assert_eq!(
                    Balances::free_balance(&other.who),
                    initial_balance - (2 * other.value / 10),
                );
            }
        });
}

#[test]
fn dont_slash_if_fraction_is_zero() {
    // Don't slash if the fraction is zero.
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Balances::free_balance(11), 1000);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(0)],
        );

        // The validator hasn't been slashed. The new era is not forced.
        assert_eq!(Balances::free_balance(11), 1000);
        assert_eq!(Staking::force_era(), Forcing::ForceNew);
    });
}


#[test]
fn only_slash_for_max_in_era() {
    // multiple slashes within one era are only applied if it is more than any previous slash in the
    // same era.
    ExtBuilder::default().build_and_execute(|| {
        assert_eq!(Balances::free_balance(11), 1000);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(50)],
        );

        // The validator has been slashed and has been force-chilled.
        assert_eq!(Balances::free_balance(11), 500);
        assert_eq!(Staking::force_era(), Forcing::ForceNew);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(25)],
        );

        // The validator has not been slashed additionally.
        assert_eq!(Balances::free_balance(11), 500);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    11,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(60)],
        );

        // The validator got slashed 10% more.
        assert_eq!(Balances::free_balance(11), 400);
    })
}

#[test]
#[ignore = "We will update Proof of Credit"]
fn garbage_collection_after_slashing() {
    // ensures that `SlashingSpans` and `SpanSlash` of an account is removed after reaping.
    ExtBuilder::default()
        .existential_deposit(2)
        .build_and_execute(|| {
            assert_eq!(Balances::free_balance(11), 256_000);

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        11,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(11), 256_000 - 25_600);
            assert!(<Staking as crate::Store>::SlashingSpans::get(&11).is_some());
            assert_eq!(
                <Staking as crate::Store>::SpanSlash::get(&(11, 0)).amount_slashed(),
                &25_600
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        11,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(100)],
            );

            // validator and nominator slash in era are garbage-collected by era change,
            // so we don't test those here.

            assert_eq!(Balances::free_balance(11), 2);
            assert_eq!(Balances::total_balance(&11), 2);

            let slashing_spans = <Staking as crate::Store>::SlashingSpans::get(&11).unwrap();
            assert_eq!(slashing_spans.iter().count(), 2);

            // reap_stash respects num_slashing_spans so that weight is accurate
            assert_noop!(
                Staking::reap_stash(Origin::none(), 11, 0),
                Error::<Test>::IncorrectSlashingSpans
            );
            assert_ok!(Staking::reap_stash(Origin::none(), 11, 2));

            assert!(<Staking as crate::Store>::SlashingSpans::get(&11).is_none());
            assert_eq!(
                <Staking as crate::Store>::SpanSlash::get(&(11, 0)).amount_slashed(),
                &0
            );
        })
}

#[test]
#[ignore = "We will update Proof of Credit"]
fn garbage_collection_on_window_pruning() {
    // ensures that `ValidatorSlashInEra` and `NominatorSlashInEra` are cleared after
    // `BondingDuration`.
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);

        assert_eq!(Balances::free_balance(11), 1000);
        let now = Staking::active_era().unwrap().index;

        let exposure = Staking::eras_stakers(now, 11);
        assert_eq!(Balances::free_balance(101), 2000);
        let nominated_value = exposure.others.iter().find(|o| o.who == 101).unwrap().value;

        on_offence_now(
            &[OffenceDetails {
                offender: (11, Staking::eras_stakers(now, 11)),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        assert_eq!(Balances::free_balance(11), 900);
        assert_eq!(Balances::free_balance(101), 2000 - (nominated_value / 10));

        assert!(<Staking as crate::Store>::ValidatorSlashInEra::get(&now, &11).is_some());
        assert!(<Staking as crate::Store>::NominatorSlashInEra::get(&now, &101).is_some());

        // + 1 because we have to exit the bonding window.
        for era in (0..(BondingDuration::get() + 1)).map(|offset| offset + now + 1) {
            assert!(<Staking as crate::Store>::ValidatorSlashInEra::get(&now, &11).is_some());
            assert!(<Staking as crate::Store>::NominatorSlashInEra::get(&now, &101).is_some());

            mock::start_active_era(era);
        }

        assert!(<Staking as crate::Store>::ValidatorSlashInEra::get(&now, &11).is_none());
        assert!(<Staking as crate::Store>::NominatorSlashInEra::get(&now, &101).is_none());
    })
}

#[test]
#[ignore = "We will update Proof of Credit"]
fn slashes_are_summed_across_spans() {
    ExtBuilder::default().build_and_execute(|| {
        mock::start_active_era(1);
        mock::start_active_era(2);
        mock::start_active_era(3);

        assert_eq!(Balances::free_balance(21), 2000);
        assert_eq!(Staking::slashable_balance_of(&21), 1000);

        let get_span = |account| <Staking as crate::Store>::SlashingSpans::get(&account).unwrap();

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    21,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        let expected_spans = vec![
            slashing::SlashingSpan {
                index: 1,
                start: 4,
                length: None,
            },
            slashing::SlashingSpan {
                index: 0,
                start: 0,
                length: Some(4),
            },
        ];

        assert_eq!(get_span(21).iter().collect::<Vec<_>>(), expected_spans);
        assert_eq!(Balances::free_balance(21), 1900);

        // 21 has been force-chilled. re-signal intent to validate.
        Staking::validate(Origin::signed(20), Default::default()).unwrap();

        mock::start_active_era(4);

        assert_eq!(Staking::slashable_balance_of(&21), 900);

        on_offence_now(
            &[OffenceDetails {
                offender: (
                    21,
                    Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                ),
                reporters: vec![],
            }],
            &[Perbill::from_percent(10)],
        );

        let expected_spans = vec![
            slashing::SlashingSpan {
                index: 2,
                start: 5,
                length: None,
            },
            slashing::SlashingSpan {
                index: 1,
                start: 4,
                length: Some(1),
            },
            slashing::SlashingSpan {
                index: 0,
                start: 0,
                length: Some(4),
            },
        ];

        assert_eq!(get_span(21).iter().collect::<Vec<_>>(), expected_spans);
        assert_eq!(Balances::free_balance(21), 1810);
    });
}

#[test]
#[ignore = "We will update Proof of Credit"]
fn deferred_slashes_are_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build_and_execute(|| {
            mock::start_active_era(1);

            assert_eq!(Balances::free_balance(11), 1000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
            assert_eq!(Balances::free_balance(101), 2000);
            let nominated_value = exposure.others.iter().find(|o| o.who == 101).unwrap().value;

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        11,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 11),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(2);

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(3);

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            // at the start of era 4, slashes from era 1 are processed,
            // after being deferred for at least 2 full eras.
            mock::start_active_era(4);

            assert_eq!(Balances::free_balance(11), 900);
            assert_eq!(Balances::free_balance(101), 2000 - (nominated_value / 10));
        })
}

#[test]
#[ignore = "We will update Proof of Credit"]
fn remove_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build_and_execute(|| {
            mock::start_active_era(1);

            assert_eq!(Balances::free_balance(11), 1000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
            assert_eq!(Balances::free_balance(101), 2000);
            let nominated_value = exposure.others.iter().find(|o| o.who == 101).unwrap().value;

            on_offence_now(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(2);

            on_offence_in_era(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(15)],
                1,
            );

            // fails if empty
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![]),
                Error::<Test>::EmptyTargets
            );

            assert_ok!(Staking::cancel_deferred_slash(Origin::root(), 1, vec![0]));

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(3);

            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            // at the start of era 4, slashes from era 1 are processed,
            // after being deferred for at least 2 full eras.
            mock::start_active_era(4);

            // the first slash for 10% was cancelled, so no effect.
            assert_eq!(Balances::free_balance(11), 1000);
            assert_eq!(Balances::free_balance(101), 2000);

            mock::start_active_era(5);

            let slash_10 = Perbill::from_percent(10);
            let slash_15 = Perbill::from_percent(15);
            let initial_slash = slash_10 * nominated_value;

            let total_slash = slash_15 * nominated_value;
            let actual_slash = total_slash - initial_slash;

            // 5% slash (15 - 10) processed now.
            assert_eq!(Balances::free_balance(11), 950);
            assert_eq!(Balances::free_balance(101), 2000 - actual_slash);
        })
}

#[test]
fn remove_multi_deferred() {
    ExtBuilder::default()
        .slash_defer_duration(2)
        .build_and_execute(|| {
            mock::start_active_era(1);

            assert_eq!(Balances::free_balance(11), 1000);

            let exposure = Staking::eras_stakers(Staking::active_era().unwrap().index, 11);
            assert_eq!(Balances::free_balance(101), 2000);

            on_offence_now(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (
                        21,
                        Staking::eras_stakers(Staking::active_era().unwrap().index, 21),
                    ),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(10)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (11, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (42, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            on_offence_now(
                &[OffenceDetails {
                    offender: (69, exposure.clone()),
                    reporters: vec![],
                }],
                &[Perbill::from_percent(25)],
            );

            assert_eq!(<Staking as Store>::UnappliedSlashes::get(&1).len(), 5);

            // fails if list is not sorted
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![2, 0, 4]),
                Error::<Test>::NotSortedAndUnique
            );
            // fails if list is not unique
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![0, 2, 2]),
                Error::<Test>::NotSortedAndUnique
            );
            // fails if bad index
            assert_noop!(
                Staking::cancel_deferred_slash(Origin::root(), 1, vec![1, 2, 3, 4, 5]),
                Error::<Test>::InvalidSlashIndex
            );

            assert_ok!(Staking::cancel_deferred_slash(
                Origin::root(),
                1,
                vec![0, 2, 4]
            ));

            let slashes = <Staking as Store>::UnappliedSlashes::get(&1);
            assert_eq!(slashes.len(), 2);
            assert_eq!(slashes[0].validator, 21);
            assert_eq!(slashes[1].validator, 42);
        })
}
*/

#[test]
fn six_session_delay() {
    ExtBuilder::default()
        .initialize_first_session(false)
        .build_and_execute(|| {
            use pallet_session::SessionManager;

            let val_set = Session::validators();
            let init_session = Session::current_index();
            let init_active_era = Staking::active_era().unwrap().index;

            // pallet-session is delaying session by one, thus the next session to plan is +2.
            assert_eq!(
                <Staking as SessionManager<_>>::new_session(init_session + 2),
                None
            );
            assert_eq!(
                <Staking as SessionManager<_>>::new_session(init_session + 3),
                Some(val_set.clone())
            );
            assert_eq!(
                <Staking as SessionManager<_>>::new_session(init_session + 4),
                None
            );
            assert_eq!(
                <Staking as SessionManager<_>>::new_session(init_session + 5),
                None
            );
            assert_eq!(
                <Staking as SessionManager<_>>::new_session(init_session + 6),
                Some(val_set.clone())
            );

            <Staking as SessionManager<_>>::end_session(init_session);
            <Staking as SessionManager<_>>::start_session(init_session + 1);
            assert_eq!(active_era(), init_active_era);

            <Staking as SessionManager<_>>::end_session(init_session + 1);
            <Staking as SessionManager<_>>::start_session(init_session + 2);
            assert_eq!(active_era(), init_active_era);

            // Reward current era
            Staking::reward_by_ids(vec![(11, 1)]);

            // New active era is triggered here.
            <Staking as SessionManager<_>>::end_session(init_session + 2);
            <Staking as SessionManager<_>>::start_session(init_session + 3);
            assert_eq!(active_era(), init_active_era + 1);

            <Staking as SessionManager<_>>::end_session(init_session + 3);
            <Staking as SessionManager<_>>::start_session(init_session + 4);
            assert_eq!(active_era(), init_active_era + 1);

            <Staking as SessionManager<_>>::end_session(init_session + 4);
            <Staking as SessionManager<_>>::start_session(init_session + 5);
            assert_eq!(active_era(), init_active_era + 1);

            // Reward current era
            Staking::reward_by_ids(vec![(21, 2)]);

            // New active era is triggered here.
            <Staking as SessionManager<_>>::end_session(init_session + 5);
            <Staking as SessionManager<_>>::start_session(init_session + 6);
            assert_eq!(active_era(), init_active_era + 2);

            // That reward are correct
            assert_eq!(Staking::eras_reward_points(init_active_era).total, 1);
            assert_eq!(Staking::eras_reward_points(init_active_era + 1).total, 2);
        });
}

#[test]
fn bond_during_era_correctly_populates_claimed_rewards() {
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            // Era = None
            bond_validator(9, 8, 1000);
            assert_eq!(
                Staking::ledger(&8),
                Some(StakingLedger {
                    stash: 9,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![],
                })
            );
            mock::start_active_era(5);
            bond_validator(11, 10, 1000);
            assert_eq!(
                Staking::ledger(&10),
                Some(StakingLedger {
                    stash: 11,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: (0..5).collect(),
                })
            );
            mock::start_active_era(99);
            bond_validator(13, 12, 1000);
            assert_eq!(
                Staking::ledger(&12),
                Some(StakingLedger {
                    stash: 13,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: (15..99).collect(),
                })
            );
        });
}

#[test]
fn on_initialize_weight_is_correct() {
    ExtBuilder::default()
        .has_stakers(false)
        .build_and_execute(|| {
            assert_eq!(Validators::<Test>::iter().count(), 0);
            // When this pallet has nothing, we do 4 reads each block
            let base_weight = <Test as frame_system::Config>::DbWeight::get().reads(4);
            assert_eq!(base_weight, Staking::on_initialize(0));
        });
}

#[test]
fn session_buffering_with_offset() {
    // similar to live-chains, have some offset for the first session
    ExtBuilder::default()
        .offset(2)
        .period(5)
        .session_per_era(5)
        .build_and_execute(|| {
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 0);

            start_session(1);
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 1);
            assert_eq!(System::block_number(), 2);

            start_session(2);
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 2);
            assert_eq!(System::block_number(), 7);

            start_session(3);
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 3);
            assert_eq!(System::block_number(), 12);

            // active era is lagging behind by one session, because of how session module works.
            start_session(4);
            assert_eq!(current_era(), 1);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 4);
            assert_eq!(System::block_number(), 17);

            start_session(5);
            assert_eq!(current_era(), 1);
            assert_eq!(active_era(), 1);
            assert_eq!(Session::current_index(), 5);
            assert_eq!(System::block_number(), 22);

            // go all the way to active 2.
            start_active_era(2);
            assert_eq!(current_era(), 2);
            assert_eq!(active_era(), 2);
            assert_eq!(Session::current_index(), 10);
        });
}

#[test]
fn session_buffering_no_offset() {
    // no offset, first session starts immediately
    ExtBuilder::default()
        .offset(0)
        .period(5)
        .session_per_era(5)
        .build_and_execute(|| {
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 0);

            start_session(1);
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 1);
            assert_eq!(System::block_number(), 5);

            start_session(2);
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 2);
            assert_eq!(System::block_number(), 10);

            start_session(3);
            assert_eq!(current_era(), 0);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 3);
            assert_eq!(System::block_number(), 15);

            // active era is lagging behind by one session, because of how session module works.
            start_session(4);
            assert_eq!(current_era(), 1);
            assert_eq!(active_era(), 0);
            assert_eq!(Session::current_index(), 4);
            assert_eq!(System::block_number(), 20);

            start_session(5);
            assert_eq!(current_era(), 1);
            assert_eq!(active_era(), 1);
            assert_eq!(Session::current_index(), 5);
            assert_eq!(System::block_number(), 25);

            // go all the way to active 2.
            start_active_era(2);
            assert_eq!(current_era(), 2);
            assert_eq!(active_era(), 2);
            assert_eq!(Session::current_index(), 10);
        });
}

#[test]
fn cannot_rebond_to_lower_than_ed() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // stash must have more balance than bonded for this to work.
            assert_eq!(Balances::free_balance(&21), 512_000);

            // initial stuff.
            assert_eq!(
                Staking::ledger(&20).unwrap(),
                StakingLedger {
                    stash: 21,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![]
                }
            );

            // unbond all of it.
            assert_ok!(Staking::unbond(Origin::signed(20), 1000));
            assert_eq!(
                Staking::ledger(&20).unwrap(),
                StakingLedger {
                    stash: 21,
                    total: 1000,
                    active: 0,
                    unlocking: vec![UnlockChunk {
                        value: 1000,
                        era: 3
                    }],
                    claimed_rewards: vec![]
                }
            );

            // now bond a wee bit more
            assert_noop!(
                Staking::rebond(Origin::signed(20), 5),
                Error::<Test>::InsufficientValue,
            );
        })
}

#[test]
fn cannot_bond_extra_to_lower_than_ed() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // stash must have more balance than bonded for this to work.
            assert_eq!(Balances::free_balance(&21), 512_000);

            // initial stuff.
            assert_eq!(
                Staking::ledger(&20).unwrap(),
                StakingLedger {
                    stash: 21,
                    total: 1000,
                    active: 1000,
                    unlocking: vec![],
                    claimed_rewards: vec![]
                }
            );

            // unbond all of it.
            assert_ok!(Staking::unbond(Origin::signed(20), 1000));
            assert_eq!(
                Staking::ledger(&20).unwrap(),
                StakingLedger {
                    stash: 21,
                    total: 1000,
                    active: 0,
                    unlocking: vec![UnlockChunk {
                        value: 1000,
                        era: 3
                    }],
                    claimed_rewards: vec![]
                }
            );

            // now bond a wee bit more
            assert_noop!(
                Staking::bond_extra(Origin::signed(21), 5),
                Error::<Test>::InsufficientValue,
            );
        })
}

#[test]
fn delegate() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // initialize candidate list
            <CandidateValidators<Test>>::put(vec![4, 6, 8, 10]);

            // TEST1： delegate to one validator
            // initialize credit score
            let micropayment_vec = vec![(1, 80 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec);
            assert_eq!(Credit::get_credit_score(&1), Some(105));
            // delegate credit score
            assert_ok!(Staking::delegate(Origin::signed(1), vec![4]));
            // check delegated info
            let info = Staking::delegated_to_validators(1);
            assert_eq!(info.score, 105);
            assert_eq!(info.validators, vec![4]);
            assert_eq!(Staking::candidate_delegators(4), vec![(1, 105)]);

            // TEST2： delegate to many validators
            // initialize credit score
            let micropayment_vec = vec![(11, 65 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec);
            assert_eq!(Credit::get_credit_score(&11), Some(105));
            // delegate credit score
            assert_ok!(Staking::delegate(Origin::signed(11), vec![4, 6, 8, 10]));
            // check delegated info
            let info = Staking::delegated_to_validators(11);
            assert_eq!(info.score, 105);
            assert_eq!(info.validators, vec![4, 6, 8, 10]);
            assert_eq!(Staking::candidate_delegators(4), vec![(1, 105), (11, 27)]);
            assert_eq!(Staking::candidate_delegators(6), vec![(11, 26)]);
            assert_eq!(Staking::candidate_delegators(8), vec![(11, 26)]);
            assert_eq!(Staking::candidate_delegators(10), vec![(11, 26)]);

            //  TEST3： delegate with invalid validator
            let micropayment_vec = vec![(19, 80 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec);
            assert_noop!(
                Staking::delegate(Origin::signed(19), vec![5]),
                Error::<Test>::NotInCandidateValidator
            );

            //  TEST4： delegate with invalid validator
            assert_noop!(
                Staking::delegate(Origin::signed(19), vec![4, 5]),
                Error::<Test>::NotInCandidateValidator
            );

            //  TEST5： delegate with low score
            assert_noop!(
                Staking::delegate(Origin::signed(21), vec![4, 6]),
                Error::<Test>::CreditScoreTooLow
            );

            //  TEST6： delegate after having called delegate()
            assert_ok!(Staking::delegate(Origin::signed(22), vec![4, 6, 8, 10]));
            assert_noop!(
                Staking::delegate(Origin::signed(22), vec![4]),
                Error::<Test>::AlreadyDelegated
            );
        });
}

#[test]
fn test_undelegate() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // initialize candidate list
            <CandidateValidators<Test>>::put(vec![4, 6, 8, 10]);

            // TEST1： undelegate
            // initialize credit score
            let micropayment_vec = vec![(11, 80 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec);
            assert_eq!(Credit::get_credit_score(&11), Some(105));
            // delegate credit score
            assert_ok!(Staking::delegate(Origin::signed(11), vec![4]));
            // undelegate after calling delegate()
            assert_ok!(Staking::undelegate(Origin::signed(11)));

            // TEST2: undelegate before calling delegate()
            assert_noop!(
                Staking::undelegate(Origin::signed(12)),
                Error::<Test>::NotDelegate
            );
        });
}

#[test]
fn test_set_current_era_validators() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            Staking::set_current_era_validators(vec![4, 6, 8, 10]);
            assert_eq!(Staking::current_era_validators(), Some(vec![4, 6, 8, 10]));
        });
}

#[test]
fn test_set_candidates() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            <CandidateValidators<Test>>::put(vec![4, 6, 8, 10]);
            assert_eq!(Staking::candidate_validators(), Some(vec![4, 6, 8, 10]));
        });
}

#[test]
fn test_total_delegated_score() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            // 11 , 21 is validator in mock.rs
            let micropayment_vec1 = vec![(1, 3 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec1);
            assert_ok!(Staking::delegate(Origin::signed(1), vec![11, 21]));

            let micropayment_vec2 = vec![(2, 2 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec2);
            assert_ok!(Staking::delegate(Origin::signed(2), vec![11, 21]));

            // check total score
            mock::start_active_era(4);
            assert_eq!(
                Staking::total_delegated_score(4),
                Some(103 + 102 + 100 + 100)
            );
        });
}

#[test]
fn test_get_total_validator_score() {
    ExtBuilder::default()
        .existential_deposit(10)
        .validator_pool(true)
        .build_and_execute(|| {
            let micropayment_vec1 = vec![(1, 1 * 1_000_000_000_000_000, 1)];
            Credit::update_credit(micropayment_vec1);
            assert_ok!(Staking::delegate(Origin::signed(1), vec![11, 21]));

            let micropayment_vec2 = vec![(2, 2 * 1_000_000_000_000_000, 2)];
            Credit::update_credit(micropayment_vec2);
            assert_ok!(Staking::delegate(Origin::signed(2), vec![11, 21]));

            // check total score
            mock::start_active_era(4);
            assert_eq!(
                Staking::total_delegated_score(4),
                Some(101 + 102 + 100 + 100)
            );

            mock::start_active_era(5);
            // check total delegated score for validator
            assert_eq!(
                Staking::get_total_validator_score(Staking::current_era().unwrap(), 11),
                Some(102 + 100) // 1, 10
            );

            assert_eq!(
                Staking::get_total_validator_score(Staking::current_era().unwrap(), 21),
                Some(101 + 100) // 2, 20
            );
        });
}

#[test]
fn poc_slash() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build_and_execute(|| {
            let micropayment_vec = vec![(11, 1 * 1_000_000_000_000_000, 5)];
            Credit::update_credit(micropayment_vec);
            assert_eq!(Credit::get_credit_score(&11), Some(100 + 1)); // init score 100 plus delta 1
            assert_ok!(Staking::delegate(Origin::signed(11), vec![11, 21]));

            mock::start_active_era(4);

            Staking::poc_slash(&11, 4);
            assert_eq!(Credit::get_credit_score(&11), Some(96)); // slashed 5
        });
}
