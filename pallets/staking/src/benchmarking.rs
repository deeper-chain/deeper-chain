// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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

//! Staking pallet benchmarking.

use super::*;
use crate::Pallet as Staking;
use testing_utils::*;

pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::{traits::One, Percent};

const SEED: u32 = 0;
const MAX_SPANS: u32 = 100;
const MAX_SLASHES: u32 = 1000;
const MAX_DELEGATORS: u32 = 1_000_000;
const MAX_DELEGATES: u32 = 1;
const MAX_VALIDATORS: u32 = 1000;

// Add slashing spans to a user account. Not relevant for actual use, only to benchmark
// read and write operations.
fn add_slashing_spans<T: Config>(who: &T::AccountId, spans: u32) {
    if spans == 0 {
        return;
    }

    // For the first slashing span, we initialize
    let mut slashing_spans = crate::slashing::SlashingSpans::new(0);
    SpanSlash::<T>::insert((who, 0), crate::slashing::SpanRecord::default());

    for i in 1..spans {
        assert!(slashing_spans.end_span(i));
        SpanSlash::<T>::insert((who, i), crate::slashing::SpanRecord::default());
    }
    SlashingSpans::<T>::insert(who, slashing_spans);
}

const USER_SEED: u32 = 999666;

benchmarks! {
    where_clause { where T: Config, T: pallet_credit::Config }
    bond {
        let stash = create_funded_user::<T>("stash", USER_SEED, 100);
        let controller = create_funded_user::<T>("controller", USER_SEED, 100);
        let controller_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(controller.clone());
        let reward_destination = RewardDestination::Staked;
        let amount = <T as Config>::Currency::minimum_balance() * 10u32.into();
        whitelist_account!(stash);
    }: _(RawOrigin::Signed(stash.clone()), controller_lookup, amount, reward_destination)
    verify {
        assert!(Bonded::<T>::contains_key(stash));
        assert!(Ledger::<T>::contains_key(controller));
    }

    bond_extra {
        let (stash, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        let max_additional = <T as Config>::Currency::minimum_balance() * 10u32.into();
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created before")?;
        let original_bonded: BalanceOf<T> = ledger.active;
        whitelist_account!(stash);
    }: _(RawOrigin::Signed(stash), max_additional)
    verify {
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created after")?;
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded < new_bonded);
    }

    unbond {
        let (_, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        let amount = <T as Config>::Currency::minimum_balance() * 10u32.into();
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created before")?;
        let original_bonded: BalanceOf<T> = ledger.active;
        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller.clone()), amount)
    verify {
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created after")?;
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded > new_bonded);
    }

    // Withdraw only updates the ledger
    withdraw_unbonded_update {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        let (stash, controller) = create_stash_controller::<T>(0, 100, Default::default())?;
        add_slashing_spans::<T>(&stash, s);
        let amount = <T as Config>::Currency::minimum_balance() * 5u32.into(); // Half of total
        Staking::<T>::unbond(RawOrigin::Signed(controller.clone()).into(), amount)?;
        CurrentEra::put(EraIndex::max_value());
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created before")?;
        let original_total: BalanceOf<T> = ledger.total;
        whitelist_account!(controller);
    }: withdraw_unbonded(RawOrigin::Signed(controller.clone()), s)
    verify {
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created after")?;
        let new_total: BalanceOf<T> = ledger.total;
        assert!(original_total > new_total);
    }

    // Worst case scenario, everything is removed after the bonding duration
    withdraw_unbonded_kill {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        let (stash, controller) = create_stash_controller::<T>(0, 100, Default::default())?;
        add_slashing_spans::<T>(&stash, s);
        let amount = <T as Config>::Currency::minimum_balance() * 10u32.into();
        Staking::<T>::unbond(RawOrigin::Signed(controller.clone()).into(), amount)?;
        CurrentEra::put(EraIndex::max_value());
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created before")?;
        let original_total: BalanceOf<T> = ledger.total;
        whitelist_account!(controller);
    }: withdraw_unbonded(RawOrigin::Signed(controller.clone()), s)
    verify {
        assert!(!Ledger::<T>::contains_key(controller));
    }

    validate {
        let (stash, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        let prefs = ValidatorPrefs::default();
        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller), prefs)
    verify {
        assert!(Validators::<T>::contains_key(stash));
    }

    // Worst case scenario, MAX_DELEGATORS
    delegate {
        let n in 1 .. MAX_DELEGATORS;
        let delegator = create_delegator::<T>(n + 1, 100)?;
        let validators = create_validators_is_accountid::<T>(MAX_DELEGATES, 100)?;
        whitelist_account!(delegator);
    }: _(RawOrigin::Signed(delegator.clone()), validators)
    verify {
        assert!(Delegators::<T>::contains_key(delegator));
    }

    undelegate {
        let delegator = create_delegator::<T>(1, 100)?;
        let validators = create_validators_is_accountid::<T>(MAX_DELEGATES, 100)?;
        Staking::<T>::delegate(RawOrigin::Signed(delegator.clone()).into(), validators)?;
        whitelist_account!(delegator);
    }: _(RawOrigin::Signed(delegator.clone()))
    verify {
        assert!(!Delegators::<T>::get(delegator).delegating);
    }

    chill {
        let (_, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller))

    set_payee {
        let (stash, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        assert_eq!(Payee::<T>::get(&stash), RewardDestination::Staked);
        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller), RewardDestination::Controller)
    verify {
        assert_eq!(Payee::<T>::get(&stash), RewardDestination::Controller);
    }

    set_controller {
        let (stash, _) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        let new_controller = create_funded_user::<T>("new_controller", USER_SEED, 100);
        let new_controller_lookup = T::Lookup::unlookup(new_controller.clone());
        whitelist_account!(stash);
    }: _(RawOrigin::Signed(stash), new_controller_lookup)
    verify {
        assert!(Ledger::<T>::contains_key(&new_controller));
    }

    set_era_validator_reward {
        let amount = <T as Config>::Currency::minimum_balance() * 100000u32.into();
    }: _(RawOrigin::Root, amount)
    verify {
        assert_eq!(EraValidatorReward::<T>::get(), amount);
    }

    set_validator_count {
        let validator_count = MAX_VALIDATORS;
    }: _(RawOrigin::Root, validator_count)
    verify {
        assert_eq!(ValidatorCount::get(), validator_count);
    }

    increase_validator_count {
        let n in 1 .. MAX_VALIDATORS;
        let pre_v_number = ValidatorCount::get();
    }: _(RawOrigin::Root, n)
    verify {
        assert_eq!(ValidatorCount::get(), pre_v_number + n);
    }

    scale_validator_count {
        let n in 1 .. 100;
        let pre_v_number = ValidatorCount::get();
        let factor = Percent::from_rational_approximation(n,100);
    }: _(RawOrigin::Root, factor)
    verify {
        assert_eq!(ValidatorCount::get(), pre_v_number + factor * pre_v_number);
    }

    force_no_eras {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::get(), Forcing::ForceNone); }

    force_new_era {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::get(), Forcing::ForceNew); }

    force_new_era_always {}: _(RawOrigin::Root)
    verify { assert_eq!(ForceEra::get(), Forcing::ForceAlways); }

    // Worst case scenario, the list of invulnerables is very long.
    set_invulnerables {
        let v in 0 .. MAX_VALIDATORS;
        let mut invulnerables = Vec::new();
        for i in 0 .. v {
            invulnerables.push(account("invulnerable", i, SEED));
        }
    }: _(RawOrigin::Root, invulnerables)
    verify {
        assert_eq!(Invulnerables::<T>::get().len(), v as usize);
    }

    set_validator_whitelist {
        let v in 0 .. MAX_VALIDATORS;
        let mut validators = Vec::new();
        for i in 0 .. v {
            validators.push(account("invulnerable", i, SEED));
        }
    }: _(RawOrigin::Root, validators)
    verify {
        assert_eq!(ValidatorWhiteList::<T>::get().len(), v as usize);
    }

    force_unstake {
        // Slashing Spans
        let s in 0 .. MAX_SPANS;
        let (stash, controller) = create_stash_controller::<T>(0, 100, Default::default())?;
        add_slashing_spans::<T>(&stash, s);
    }: _(RawOrigin::Root, stash, s)
    verify {
        assert!(!Ledger::<T>::contains_key(&controller));
    }

    increase_mining_reward {
        let r in 0 .. 10000;
        let remainder = Staking::<T>::remainder_mining_reward().unwrap_or(T::TotalMiningReward::get());
    }: _(RawOrigin::Root, r as u128)
    verify {
        assert_eq!(remainder + r as u128, Staking::<T>::remainder_mining_reward().unwrap_or(T::TotalMiningReward::get()));
    }

    cancel_deferred_slash {
        let s in 1 .. MAX_SLASHES;
        let mut unapplied_slashes = Vec::new();
        let era = EraIndex::one();
        for _ in 0 .. MAX_SLASHES {
            unapplied_slashes.push(UnappliedSlash::<T::AccountId, BalanceOf<T>>::default());
        }
        UnappliedSlashes::<T>::insert(era, &unapplied_slashes);

        let slash_indices: Vec<u32> = (0 .. s).collect();
    }: _(RawOrigin::Root, era, slash_indices)
    verify {
        assert_eq!(UnappliedSlashes::<T>::get(&era).len(), (MAX_SLASHES - s) as usize);
    }

    rebond {
        let l in 1 .. MAX_UNLOCKING_CHUNKS as u32;
        let (_, controller) = create_stash_controller::<T>(USER_SEED, 100, Default::default())?;
        let mut staking_ledger = Ledger::<T>::get(controller.clone()).unwrap();
        let unlock_chunk = UnlockChunk::<BalanceOf<T>> {
            value: 1u32.into(),
            era: EraIndex::zero(),
        };
        for _ in 0 .. l {
            staking_ledger.unlocking.push(unlock_chunk.clone())
        }
        Ledger::<T>::insert(controller.clone(), staking_ledger.clone());
        let original_bonded: BalanceOf<T> = staking_ledger.active;
        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller.clone()), (l + 100).into())
    verify {
        let ledger = Ledger::<T>::get(&controller).ok_or("ledger not created after")?;
        let new_bonded: BalanceOf<T> = ledger.active;
        assert!(original_bonded < new_bonded);
    }

    set_history_depth {
        let e in 1 .. 100;
        HistoryDepth::put(e);
        CurrentEra::put(e);
        for i in 0 .. e {
            <ErasStakers<T>>::insert(i, T::AccountId::default(), Exposure::<T::AccountId, BalanceOf<T>>::default());
            <ErasValidatorPrefs<T>>::insert(i, T::AccountId::default(), ValidatorPrefs::default());
            <ErasRewardPoints<T>>::insert(i, EraRewardPoints::<T::AccountId>::default());
            <ErasTotalStake<T>>::insert(i, BalanceOf::<T>::one());
            ErasStartSessionIndex::insert(i, i);
        }
    }: _(RawOrigin::Root, EraIndex::zero(), u32::max_value())
    verify {
        assert_eq!(HistoryDepth::get(), 0);
    }

    reap_stash {
        let s in 1 .. MAX_SPANS;
        let (stash, controller) = create_stash_controller::<T>(0, 100, Default::default())?;
        add_slashing_spans::<T>(&stash, s);
        <T as Config>::Currency::make_free_balance_be(&stash, <T as Config>::Currency::minimum_balance());
        whitelist_account!(controller);
    }: _(RawOrigin::Signed(controller), stash.clone(), s)
    verify {
        assert!(!Bonded::<T>::contains_key(&stash));
    }

    new_era {
        let v in 1 .. 10;
        let d in 1 .. 1000;

        create_validators_with_delegators_for_era::<T>(v, d, MAX_DELEGATES as usize, false, None)?;
        let session_index = SessionIndex::one();
    }: {
        let validators = Staking::<T>::new_era(session_index).ok_or("`new_era` failed")?;
        assert!(validators.len() == v as usize);
    }

    #[extra]
    do_slash {
        let l in 1 .. MAX_UNLOCKING_CHUNKS as u32;
        let (stash, controller) = create_stash_controller::<T>(0, 100, Default::default())?;
        let mut staking_ledger = Ledger::<T>::get(controller.clone()).unwrap();
        let unlock_chunk = UnlockChunk::<BalanceOf<T>> {
            value: 1u32.into(),
            era: EraIndex::zero(),
        };
        for _ in 0 .. l {
            staking_ledger.unlocking.push(unlock_chunk.clone())
        }
        Ledger::<T>::insert(controller, staking_ledger);
        let slash_amount = <T as Config>::Currency::minimum_balance() * 10u32.into();
        let balance_before = <T as Config>::Currency::free_balance(&stash);
    }: {
        crate::slashing::do_slash::<T>(
            &stash,
            slash_amount,
            &mut BalanceOf::<T>::zero(),
            &mut NegativeImbalanceOf::<T>::zero()
        );
    } verify {
        let balance_after = <T as Config>::Currency::free_balance(&stash);
        assert!(balance_before > balance_after);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Staking, Test};
    use frame_support::assert_ok;

    #[test]
    fn create_validators_with_delegators_for_era_works() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                let v = 10;
                let d = 100;

                create_validators_with_delegators_for_era::<Test>(
                    v,
                    d,
                    MAX_DELEGATES as usize,
                    false,
                    None,
                )
                .unwrap();

                let count_validators = Validators::<Test>::iter().count();
                let count_delegators = Delegators::<Test>::iter().count();

                assert_eq!(count_validators, v as usize);
                assert_eq!(count_delegators, d as usize);
            });
    }

    #[test]
    fn add_slashing_spans_works() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                let n = 10;

                if let Ok((validator_stash, _)) =
                    create_stash_controller::<Test>(n, 100, Default::default())
                {
                    // Add 20 slashing spans
                    let num_of_slashing_spans = 20;
                    add_slashing_spans::<Test>(&validator_stash, num_of_slashing_spans);

                    let slashing_spans = SlashingSpans::<Test>::get(&validator_stash).unwrap();
                    assert_eq!(
                        slashing_spans.iter().count(),
                        num_of_slashing_spans as usize
                    );
                    for i in 0..num_of_slashing_spans {
                        assert!(SpanSlash::<Test>::contains_key((&validator_stash, i)));
                    }

                    // Test everything is cleaned up
                    assert_ok!(Staking::kill_stash(&validator_stash, num_of_slashing_spans));
                    assert!(SlashingSpans::<Test>::get(&validator_stash).is_none());
                    for i in 0..num_of_slashing_spans {
                        assert!(!SpanSlash::<Test>::contains_key((&validator_stash, i)));
                    }
                }
            });
    }

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default()
            .has_stakers(true)
            .build()
            .execute_with(|| {
                assert_ok!(Pallet::<Test>::test_benchmark_bond());
                assert_ok!(Pallet::<Test>::test_benchmark_bond_extra());
                assert_ok!(Pallet::<Test>::test_benchmark_unbond());
                assert_ok!(Pallet::<Test>::test_benchmark_withdraw_unbonded_update());
                assert_ok!(Pallet::<Test>::test_benchmark_withdraw_unbonded_kill());
                assert_ok!(Pallet::<Test>::test_benchmark_validate());
                assert_ok!(Pallet::<Test>::test_benchmark_chill());
                assert_ok!(Pallet::<Test>::test_benchmark_set_payee());
                assert_ok!(Pallet::<Test>::test_benchmark_set_controller());
                assert_ok!(Pallet::<Test>::test_benchmark_set_validator_count());
                assert_ok!(Pallet::<Test>::test_benchmark_increase_validator_count());
                assert_ok!(Pallet::<Test>::test_benchmark_scale_validator_count());
                assert_ok!(Pallet::<Test>::test_benchmark_force_no_eras());
                assert_ok!(Pallet::<Test>::test_benchmark_force_new_era());
                assert_ok!(Pallet::<Test>::test_benchmark_set_invulnerables());
                assert_ok!(Pallet::<Test>::test_benchmark_set_validator_whitelist());
                assert_ok!(Pallet::<Test>::test_benchmark_force_unstake());
                assert_ok!(Pallet::<Test>::test_benchmark_force_new_era_always());
                assert_ok!(Pallet::<Test>::test_benchmark_increase_mining_reward());
                assert_ok!(Pallet::<Test>::test_benchmark_cancel_deferred_slash());
                assert_ok!(Pallet::<Test>::test_benchmark_rebond());
                assert_ok!(Pallet::<Test>::test_benchmark_set_history_depth());
                assert_ok!(Pallet::<Test>::test_benchmark_reap_stash());
                assert_ok!(Pallet::<Test>::test_benchmark_delegate());
                assert_ok!(Pallet::<Test>::test_benchmark_undelegate());
                assert_ok!(Pallet::<Test>::test_benchmark_do_slash());
                assert_ok!(Pallet::<Test>::test_benchmark_new_era());
            });
    }
}
