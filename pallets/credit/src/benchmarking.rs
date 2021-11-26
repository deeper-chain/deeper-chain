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
pub use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::Percent;

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;

benchmarks! {
    update_credit_setting {
        let credit_setting = CreditSetting::<BalanceOf<T>> {
            campaign_id: 0,
            credit_level: CreditLevel::One,
            staking_balance: 20_000u32.into(),
            base_apy: Percent::from_percent(39),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 1u32,
            tax_rate: Percent::from_percent(10),
            max_referees_with_rewards: 1,
            reward_per_referee: 18u32.into(),
        };
    }: _(RawOrigin::Root, credit_setting)
    verify {
        assert!(CreditSettings::<T>::contains_key(0, CreditLevel::One));
    }

    add_or_update_credit_data {
        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        let user: T::AccountId = account("user", USER_SEED, SEED);
    }: _(RawOrigin::Root, user.clone(), credit_data)
    verify {
        assert!(UserCredit::<T>::contains_key(user));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(Pallet::<Test>::test_benchmark_update_credit_setting());
            assert_ok!(Pallet::<Test>::test_benchmark_update_credit_setting());
        });
    }
}
