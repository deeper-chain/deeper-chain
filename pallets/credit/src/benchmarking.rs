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
use crate::Pallet as Credit;
pub use frame_benchmarking::{account, benchmarks, vec, whitelist_account, whitelisted_caller};
use frame_support::assert_ok;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use node_primitives::{
    credit::{CreditData, CreditLevel, CreditSetting},
    user_privileges::Privilege,
};
use sp_runtime::{traits::StaticLookup, Percent};

const SEED: u32 = 0;
const USER_SEED: u32 = 999666;

pub fn create_funded_user<T: Config>(
    string: &'static str,
    n: u32,
    balance_factor: u32,
) -> T::AccountId {
    let user = account(string, n, SEED);
    let balance = <T as pallet::Config>::Currency::minimum_balance() * balance_factor.into();
    <T as pallet::Config>::Currency::make_free_balance_be(&user, balance);
    <T as pallet::Config>::Currency::issue(balance);
    user
}

benchmarks! {
    where_clause {where T::ClassId: From<u16>, T::InstanceId: From<u16>,T: pallet_user_privileges::Config}

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

    burn_for_add_credit {
        let mut credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };
        let user = create_funded_user::<T>("user",USER_SEED, 1000);
        UserCredit::<T>::insert(&user,credit_data.clone());
        credit_data.credit = 101;
        UserCreditHistory::<T>::insert(&user,vec![(1,credit_data)]);
    }: _(RawOrigin::Signed(user.clone()), 1)
    verify {
        assert_eq!(UserCredit::<T>::get(&user).unwrap().credit,101);
    }

    force_modify_credit_history {
        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 270,
        };
        let user = create_funded_user::<T>("user",USER_SEED, 1000);
        UserCredit::<T>::insert(&user,credit_data.clone());
        UserCreditHistory::<T>::insert(&user,vec![(6,credit_data.clone())]);
    }: _(RawOrigin::Root, user.clone(), 7)
    verify {
        assert_eq!(UserCreditHistory::<T>::get(&user), vec![(7, credit_data)]);
    }

    update_nft_class_credit {
        let class_id: T::ClassId = 0u16.into();
        let credit = 1;
        let user = create_funded_user::<T>("user",USER_SEED, 1000);
        let user_lookup = T::Lookup::unlookup(user.clone());
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),user_lookup,Privilege::CreditAdmin);
    }: update_nft_class_credit(RawOrigin::Signed(user), class_id, credit)
    verify {
        assert_eq!(MiningMachineClassCredit::<T>::get(class_id), credit);
    }

    burn_nft {
        let class_id: T::ClassId =  0u16.into();
        let instance_id: T::InstanceId =  0u16.into();
        let user = create_funded_user::<T>("user",USER_SEED, 1000);

        let user_lookup = T::Lookup::unlookup(user.clone());
        let signed_user = RawOrigin::Signed(user.clone());
        let _ = pallet_user_privileges::Pallet::<T>::set_user_privilege(RawOrigin::Root.into(),user_lookup.clone(),Privilege::CreditAdmin);

        assert_ok!(pallet_uniques::Pallet::<T>::force_create(
            RawOrigin::Root.into(),
            class_id,
            user_lookup.clone(),
            true
        ));

        let credit_data = CreditData {
            campaign_id: 0,
            credit: 100,
            initial_credit_level: CreditLevel::One,
            rank_in_initial_credit_level: 0,
            number_of_referees: 1,
            current_credit_level: CreditLevel::One,
            reward_eras: 0,
        };

        assert_ok!(Credit::<T>::update_nft_class_credit(RawOrigin::Signed(user.clone()).into(), class_id, 5) );
        assert_ok!(Credit::<T>::add_or_update_credit_data(
            RawOrigin::Root.into(),
            user.clone(),
            credit_data.clone()
        ));

        assert_ok!(pallet_uniques::Pallet::<T>::mint(signed_user.clone().into(), class_id, instance_id, user_lookup.clone()));
    }: burn_nft(signed_user, class_id, instance_id)
    verify {
        assert_eq!(UserCredit::<T>::get(user).unwrap().credit, 105);
    }

    set_switch_campaign {

    }: _(RawOrigin::Root, vec!(0,1,2), vec!(3,4,5))
    verify {
        assert_eq!(CampaignIdSwitch::<T>::get(&0), Some(3));
    }

    set_not_switch_accounts {
        let user1: T::AccountId = account("user1", USER_SEED, SEED);
        let user2: T::AccountId = account("user2", USER_SEED, SEED);
       }: _(RawOrigin::Root, vec!(user1.clone(),user2.clone()))
       verify {
           assert_eq!(NotSwitchAccounts::<T>::get(&user1), Some(true));
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
            assert_ok!(Pallet::<Test>::test_benchmark_add_or_update_credit_data());
            assert_ok!(Pallet::<Test>::test_benchmark_burn_for_add_credit());
            assert_ok!(Pallet::<Test>::test_benchmark_force_modify_credit_history());
            assert_ok!(Pallet::<Test>::test_benchmark_update_nft_class_credit());
            assert_ok!(Pallet::<Test>::test_benchmark_burn_nft());
        });
    }
}
