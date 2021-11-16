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

//! Testing utils for staking. Provides some common functions to setup staking state, such as
//! bonding validators, nominators, and generating different types of solutions.

use crate::*;
use crate::{Config as StakingConfig, Module as Staking};
use frame_benchmarking::account;
use frame_system::RawOrigin;
use pallet_credit::{Config as CreditConfig, CreditData, CreditLevel, Module as Credit};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use sp_io::hashing::blake2_256;
use sp_std::cmp;

const SEED: u32 = 0;
const MAX_VALIDATORS: u32 = 1000;

/// This function removes all validators and delegators from storage.
pub fn clear_validators_and_delegators<T: Config>() {
    Validators::<T>::remove_all(None);
    CandidateValidators::<T>::remove_all(None);
    Delegators::<T>::remove_all(None);
}

/// Grab a funded user.
pub fn create_funded_user<T: Config>(
    string: &'static str,
    n: u32,
    balance_factor: u32,
) -> T::AccountId {
    let user = account(string, n, SEED);
    let balance = T::Currency::minimum_balance() * balance_factor.into();
    T::Currency::make_free_balance_be(&user, balance);
    // ensure T::CurrencyToVote will work correctly.
    T::Currency::issue(balance);
    user
}

/// Create a stash and controller pair.
pub fn create_stash_controller<T: Config>(
    n: u32,
    balance_factor: u32,
    destination: RewardDestination<T::AccountId>,
) -> Result<(T::AccountId, T::AccountId), &'static str> {
    let stash = create_funded_user::<T>("stash", n, balance_factor);
    let controller = create_funded_user::<T>("controller", n, balance_factor);
    let controller_lookup: <T::Lookup as StaticLookup>::Source =
        T::Lookup::unlookup(controller.clone());
    let amount = T::Currency::minimum_balance() * (balance_factor / 10).max(1).into();
    Staking::<T>::bond(
        RawOrigin::Signed(stash.clone()).into(),
        controller_lookup,
        amount,
        destination,
    )?;
    return Ok((stash, controller));
}

/// Create a delegator
pub fn create_delegator<T: StakingConfig + CreditConfig>(
    n: u32,
    balance_factor: u32,
) -> Result<T::AccountId, &'static str> {
    let delegator = create_funded_user::<T>("delegator", n, balance_factor);
    let credit_data = CreditData {
        campaign_id: 0,
        credit: 100,
        initial_credit_level: CreditLevel::Zero,
        rank_in_initial_credit_level: 0u32,
        number_of_referees: 0,
        current_credit_level: CreditLevel::One,
        reward_eras: 100,
    };
    let _ = Credit::<T>::add_or_update_credit_data(
        RawOrigin::Root.into(),
        delegator.clone(),
        credit_data,
    );
    return Ok(delegator);
}

/// create `max` validators.
pub fn create_validators<T: Config>(
    max: u32,
    balance_factor: u32,
) -> Result<Vec<<T::Lookup as StaticLookup>::Source>, &'static str> {
    let mut validators: Vec<<T::Lookup as StaticLookup>::Source> = Vec::with_capacity(max as usize);
    for i in 0..max {
        let (stash, controller) =
            create_stash_controller::<T>(i, balance_factor, RewardDestination::Staked)?;
        let validator_prefs = ValidatorPrefs {
            commission: Perbill::from_percent(0),
            ..Default::default()
        };
        Staking::<T>::validate(RawOrigin::Signed(controller).into(), validator_prefs)?;
        let stash_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(stash);
        validators.push(stash_lookup);
    }
    Ok(validators)
}

/// create `max` validators, return T::AccoutId.
pub fn create_validators_is_accountid<T: Config>(
    max: u32,
    balance_factor: u32,
) -> Result<Vec<T::AccountId>, &'static str> {
    let max_validators = cmp::min(MAX_VALIDATORS, max);
    let mut validators: Vec<T::AccountId> = Vec::with_capacity(max_validators as usize);
    for i in 0..max_validators {
        let (stash, controller) =
            create_stash_controller::<T>(i, balance_factor, RewardDestination::Staked)?;
        let validator_prefs = ValidatorPrefs {
            commission: Perbill::from_percent(0),
            ..Default::default()
        };
        Staking::<T>::validate(RawOrigin::Signed(controller).into(), validator_prefs)?;
        validators.push(stash);
    }
    Ok(validators)
}

/// This function generates validators and delegators who are randomly delegate
/// `edge_per_delegator` random validators (until `to_delegate` if provided).
///
/// NOTE: This function will remove any existing validators or validators to ensure
/// we are working with a clean state.
///
/// Parameters:
/// - `validators`: number of bonded validators
/// - `delegators`: number of bonded delegators.
/// - `edge_per_delegator`: number of edge (vote) per delegator.
/// - `randomize_stake`: whether to randomize the stakes.
/// - `to_delegate`: if `Some(n)`, only the first `n` bonded validator are voted upon.
///    Else, all of them are considered and `edge_per_delegator` random validators are voted for.
///
/// Return the validators choosen to be delegated.
pub fn create_validators_with_delegators_for_era<T: Config + pallet_credit::Config>(
    validators: u32,
    delegators: u32,
    edge_per_delegator: usize,
    randomize_stake: bool,
    to_delegate: Option<u32>,
) -> Result<Vec<<T::Lookup as StaticLookup>::Source>, &'static str> {
    clear_validators_and_delegators::<T>();

    let mut validators_stash: Vec<<T::Lookup as StaticLookup>::Source> =
        Vec::with_capacity(validators as usize);
    let mut rng = ChaChaRng::from_seed(SEED.using_encoded(blake2_256));

    // Create validators
    for i in 0..validators {
        let balance_factor = if randomize_stake {
            rng.next_u32() % 255 + 10
        } else {
            100u32
        };
        let (v_stash, v_controller) =
            create_stash_controller::<T>(i, balance_factor, RewardDestination::Staked)?;
        let validator_prefs = ValidatorPrefs {
            commission: Perbill::from_percent(50),
            ..Default::default()
        };
        Staking::<T>::validate(
            RawOrigin::Signed(v_controller.clone()).into(),
            validator_prefs,
        )?;
        let stash_lookup: <T::Lookup as StaticLookup>::Source =
            T::Lookup::unlookup(v_stash.clone());
        validators_stash.push(stash_lookup.clone());
    }

    let to_delegate = to_delegate.unwrap_or(validators_stash.len() as u32) as usize;
    let validator_choosen = validators_stash[0..to_delegate].to_vec();

    // Create delegators
    for j in 0..delegators {
        let balance_factor = if randomize_stake {
            rng.next_u32() % 255 + 10
        } else {
            100u32
        };
        let delegator = create_delegator::<T>(j + 1, balance_factor)?;

        // Have them randomly validate
        let mut available_validators = validator_choosen.clone();
        let mut selected_validators: Vec<T::AccountId> = Vec::with_capacity(edge_per_delegator);

        for _ in 0..validators.min(edge_per_delegator as u32) {
            let selected = rng.next_u32() as usize % available_validators.len();
            let validator = available_validators.remove(selected);
            if let Ok(validator_acccount_id) = T::Lookup::lookup(validator) {
                selected_validators.push(validator_acccount_id);
            }
        }
        Staking::<T>::delegate(
            RawOrigin::Signed(delegator.clone()).into(),
            selected_validators,
        )?;
    }

    ValidatorCount::put(validators);

    Ok(validator_choosen)
}
