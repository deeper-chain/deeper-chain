// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020 Parity Technologies (UK) Ltd.
//
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

#![cfg(test)]

use super::*;
use crate::mock::*;

use frame_support::{assert_ok, traits::GenesisBuild};
use std::{collections::BTreeMap, str::FromStr};

type Balances = pallet_balances::Pallet<Test>;
type EVM = Pallet<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut account_pairs = BTreeMap::new();
    account_pairs.insert(
        H160::from_str("1000000000000000000000000000000000000001").unwrap(),
        AccountId32::new([1u8; 32]),
    );
    account_pairs.insert(
        H160::from_str("1000000000000000000000000000000000000002").unwrap(),
        AccountId32::new([2u8; 32]),
    );
    account_pairs.insert(
        H160::from_str("1234500000000000000000000000000000000000").unwrap(),
        AccountId32::new([3u8; 32]),
    );
    account_pairs.insert(
        H160::from_str("1000000000000000000000000000000000000003").unwrap(),
        AccountId32::new([4u8; 32]),
    );

    let mut accounts = BTreeMap::new();
    accounts.insert(
        H160::from_str("1000000000000000000000000000000000000001").unwrap(),
        GenesisAccount {
            nonce: U256::from(1),
            balance: U256::from_str("0xffffffffffffffffffffffffffffffff").unwrap(),
            storage: Default::default(),
            code: vec![
                0x00, // STOP
            ],
        },
    );
    accounts.insert(
        H160::from_str("1000000000000000000000000000000000000002").unwrap(),
        GenesisAccount {
            nonce: U256::from(1),
            balance: U256::from_str("0xffffffffffffffffffffffffffffffff").unwrap(),
            storage: Default::default(),
            code: vec![
                0xff, // INVALID
            ],
        },
    );
    accounts.insert(
        H160::default(), // root
        GenesisAccount {
            nonce: U256::from(1),
            balance: U256::max_value(),
            storage: Default::default(),
            code: vec![],
        },
    );

    pallet_balances::GenesisConfig::<Test> {
        // Create the block author account with some balance.
        balances: vec![(AccountId32::new([3u8; 32]), 12345)],
    }
    .assimilate_storage(&mut t)
    .expect("Pallet balances storage can be assimilated");
    GenesisBuild::<Test>::assimilate_storage(
        &crate::GenesisConfig {
            account_pairs,
            accounts,
        },
        &mut t,
    )
    .unwrap();
    t.into()
}

#[test]
fn fee_deduction() {
    new_test_ext().execute_with(|| {
		// Create an EVM address and the corresponding Substrate address that will be charged fees and refunded
		let evm_addr = H160::from_str("1000000000000000000000000000000000000003").unwrap();
		let substrate_addr = <Test as Config>::AddressMapping::into_account_id(evm_addr);

		// Seed account
		let _ = <Test as Config>::Currency::deposit_creating(&substrate_addr, 100);
		assert_eq!(Balances::free_balance(&substrate_addr), 100);

		// Deduct fees as 10 units
		let imbalance = <<Test as Config>::OnChargeTransaction as OnChargeEVMTransaction<Test>>::withdraw_fee(&evm_addr, U256::from(10)).unwrap();
		assert_eq!(Balances::free_balance(&substrate_addr), 90);

		// Refund fees as 5 units
		<<Test as Config>::OnChargeTransaction as OnChargeEVMTransaction<Test>>::correct_and_deposit_fee(&evm_addr, U256::from(5), imbalance);
		assert_eq!(Balances::free_balance(&substrate_addr), 95);
	});
}

#[test]
fn ed_0_refund_patch_works() {
    new_test_ext().execute_with(|| {
        // Verifies that the OnChargeEVMTransaction patch is applied and fixes a known bug in Substrate for evm transactions.
        // https://github.com/paritytech/substrate/issues/10117
        let evm_addr = H160::from_str("1000000000000000000000000000000000000003").unwrap();
        let substrate_addr = <Test as Config>::AddressMapping::into_account_id(evm_addr);

        let _ = <Test as Config>::Currency::deposit_creating(&substrate_addr, 21_777_000_000_000);
        assert_eq!(Balances::free_balance(&substrate_addr), 21_777_000_000_000);

        assert_ok!(EVM::call(
            Origin::signed(substrate_addr.clone()),
            evm_addr,
            H160::from_str("1000000000000000000000000000000000000004").unwrap(),
            Vec::new(),
            U256::from(1_000_000_000),
            21776,
            U256::from(1_000_000_000),
            None,
            Some(U256::from(0)),
            Vec::new(),
        ));

        // All that was due, was refunded.
        assert_eq!(Balances::free_balance(&substrate_addr), 776_000_000_000);
    });
}

#[test]
fn ed_0_refund_patch_is_required() {
    new_test_ext().execute_with(|| {
        // This test proves that the patch is required, verifying that the current Substrate behaviour is incorrect
        // for ED 0 configured chains.
        let evm_addr = H160::from_str("1000000000000000000000000000000000000003").unwrap();
        let substrate_addr = <Test as Config>::AddressMapping::into_account_id(evm_addr);

        let _ = <Test as Config>::Currency::deposit_creating(&substrate_addr, 100);
        assert_eq!(Balances::free_balance(&substrate_addr), 100);

        // Drain funds
        let _ =
            <<Test as Config>::OnChargeTransaction as OnChargeEVMTransaction<Test>>::withdraw_fee(
                &evm_addr,
                U256::from(100),
            )
            .unwrap();
        assert_eq!(Balances::free_balance(&substrate_addr), 0);

        // Try to refund. With ED 0, although the balance is now 0, the account still exists.
        // So its expected that calling `deposit_into_existing` results in the AccountData to increase the Balance.
        //
        // Is not the case, and this proves that the refund logic needs to be handled taking this into account.
        assert_eq!(
            <Test as Config>::Currency::deposit_into_existing(&substrate_addr, 5u32.into())
                .is_err(),
            true
        );
        // Balance didn't change, and should be 5.
        assert_eq!(Balances::free_balance(&substrate_addr), 0);
    });
}

#[test]
fn find_author() {
    new_test_ext().execute_with(|| {
        let author = EVM::find_author();
        assert_eq!(
            author,
            H160::from_str("1234500000000000000000000000000000000000").unwrap()
        );
    });
}

/* todo: EVM::account_basic(&evm_addr).balance should be free_balance or usable_balance?
#[test]
fn reducible_balance() {
    new_test_ext().execute_with(|| {
        let evm_addr = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let account_id = <Test as Config>::AddressMapping::into_account_id(evm_addr);
        let existential = ExistentialDeposit::get();

        // Genesis Balance.
        let genesis_balance = EVM::account_basic(&evm_addr).balance;

        // Lock identifier.
        let lock_id: LockIdentifier = *b"te/stlok";
        // Reserve some funds.
        let to_lock = 1000;
        Balances::set_lock(lock_id, &account_id, to_lock, WithdrawReasons::RESERVE);
        // Reducible is, as currently configured in `account_basic`, (balance - lock + existential).
        let reducible_balance = EVM::account_basic(&evm_addr).balance;
        assert_eq!(reducible_balance, (genesis_balance - to_lock + existential));
    });
}
*/

#[test]
fn author_should_get_tip() {
    new_test_ext().execute_with(|| {
        let author = EVM::find_author();
        let before_tip = EVM::account_basic(&author).balance;
        let evm_addr = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let account_id = <Test as Config>::AddressMapping::into_account_id(evm_addr);
        assert_ok!(EVM::call(
            Origin::signed(account_id),
            evm_addr,
            H160::from_str("1000000000000000000000000000000000000002").unwrap(),
            Vec::new(),
            U256::from(1),
            1000000,
            U256::from(1_000_000_000),
            Some(U256::from(1)),
            None,
            Vec::new(),
        ));
        let after_tip = EVM::account_basic(&author).balance;
        assert_eq!(after_tip, (before_tip + 21000));
    });
}

#[test]
fn author_same_balance_without_tip() {
    new_test_ext().execute_with(|| {
        let author = EVM::find_author();
        let before_tip = EVM::account_basic(&author).balance;
        let evm_addr = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let account_id = <Test as Config>::AddressMapping::into_account_id(evm_addr);
        assert_ok!(EVM::call(
            Origin::signed(account_id),
            evm_addr,
            H160::from_str("1000000000000000000000000000000000000002").unwrap(),
            Vec::new(),
            U256::default(),
            1000000,
            U256::from(1_000_000_000),
            None,
            None,
            Vec::new(),
        ));
        let after_tip = EVM::account_basic(&author).balance;
        assert_eq!(after_tip, before_tip);
    });
}

#[test]
fn refunds_should_work() {
    new_test_ext().execute_with(|| {
        // Gas price is not part of the actual fee calculations anymore, only the base fee.
        //
        // Because we first deduct max_fee_per_gas * gas_limit (2_000_000_000 * 1000000) we need
        // to ensure that the difference (max fee VS base fee) is refunded.
        let evm_addr = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let account_id = <Test as Config>::AddressMapping::into_account_id(evm_addr);
        let before_call = EVM::account_basic(&evm_addr).balance;
        assert_ok!(EVM::call(
            Origin::signed(account_id),
            evm_addr,
            H160::from_str("1000000000000000000000000000000000000003").unwrap(),
            Vec::new(),
            U256::from_str("0xfffffffffffff").unwrap(),
            1000000,
            U256::from(2_000_000_000),
            None,
            None,
            Vec::new(),
        ));
        let total_cost = (U256::from(21_000) * <Test as Config>::FeeCalculator::min_gas_price())
            + U256::from_str("0xfffffffffffff").unwrap();
        let after_call = EVM::account_basic(&evm_addr).balance;
        assert_eq!(after_call, before_call - total_cost);
    });
}

#[test]
fn refunds_and_priority_should_work() {
    new_test_ext().execute_with(|| {
        let author = EVM::find_author();
        let before_tip = EVM::account_basic(&author).balance;
        let evm_addr = H160::from_str("1000000000000000000000000000000000000001").unwrap();
        let account_id = <Test as Config>::AddressMapping::into_account_id(evm_addr);
        let before_call = EVM::account_basic(&evm_addr).balance;
        let tip = 5;
        // The tip is deducted but never refunded to the caller.
        assert_ok!(EVM::call(
            Origin::signed(account_id),
            evm_addr,
            H160::from_str("1000000000000000000000000000000000000003").unwrap(),
            Vec::new(),
            U256::from_str("0xfffffffffffff").unwrap(),
            1000000,
            U256::from(2_000_000_000),
            Some(U256::from(tip)),
            None,
            Vec::new(),
        ));
        let tip = tip * 21000;
        let total_cost = (U256::from(21_000) * <Test as Config>::FeeCalculator::min_gas_price())
            + U256::from_str("0xfffffffffffff").unwrap()
            + U256::from(tip);
        let after_call = EVM::account_basic(&evm_addr).balance;
        assert_eq!(after_call, before_call - total_cost);

        let after_tip = EVM::account_basic(&author).balance;
        assert_eq!(after_tip, (before_tip + tip));
    });
}
