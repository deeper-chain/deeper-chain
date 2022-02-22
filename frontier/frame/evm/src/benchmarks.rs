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

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{runner::Runner, Config, Pallet};
use frame_benchmarking::benchmarks;
use rlp::RlpStream;
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};
use sp_std::vec;

benchmarks! {

	// This benchmark tests the relationship between gas and weight. It deploys a contract which
	// has an infinite loop in a public function. We then call this function with varying amounts
	// of gas, expecting it to OOG. The benchmarking framework measures the amount of time (aka
	// weight) it takes before OOGing and relates that to the amount of gas provided, leaving us
	// with an estimate for gas-to-weight mapping.
	runner_execute {

		let x in 1..10000000;

		// contract bytecode below is for:
		//
		// pragma solidity >=0.8.0;
		//
		// contract InfiniteContractVar {
		//     uint public count;

		//     constructor() public {
		//         count = 0;
		//     }

		//     function infinite() public {
		//         while (true) {
		//             count=count+1;
		//         }
		//     }
		// }

		/*
		let contract_bytecode = hex::decode(concat!(
			"608060405234801561001057600080fd5b506000808190555061017c806100276000396000f3fe60",
			"8060405234801561001057600080fd5b50600436106100365760003560e01c806306661abd146100",
			"3b5780635bec9e6714610059575b600080fd5b610043610063565b604051610050919061009c565b",
			"60405180910390f35b610061610069565b005b60005481565b5b60011561008b5760016000546100",
			"8091906100b7565b60008190555061006a565b565b6100968161010d565b82525050565b60006020",
			"820190506100b1600083018461008d565b92915050565b60006100c28261010d565b91506100cd83",
			"61010d565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			"ff0382111561010257610101610117565b5b828201905092915050565b6000819050919050565b7f",
			"4e487b71000000000000000000000000000000000000000000000000000000006000526011600452",
			"60246000fdfea2646970667358221220bcab0385167dbed28dee34f1c5b30ecfcd67915495f0a32d",
			"2eeada8e094193a364736f6c63430008030033"))
			.expect("Bad hex string");
		*/
		// refer to 'deeper chain: scripts/frontier/benchmark_utils'
		let contract_bytecode = vec![96, 128, 96, 64, 82, 52, 128, 21, 97, 0, 16, 87, 96, 0, 128, 253, 91, 80, 96, 0, 128, 129, 144, 85, 80, 97, 1, 124, 128, 97, 0, 39, 96, 0, 57, 96, 0, 243, 254, 96, 128, 96, 64, 82, 52, 128, 21, 97, 0, 16, 87, 96, 0, 128, 253, 91, 80, 96, 4, 54, 16, 97, 0, 54, 87, 96, 0, 53, 96, 224, 28, 128, 99, 6, 102, 26, 189, 20, 97, 0, 59, 87, 128, 99, 91, 236, 158, 103, 20, 97, 0, 89, 87, 91, 96, 0, 128, 253, 91, 97, 0, 67, 97, 0, 99, 86, 91, 96, 64, 81, 97, 0, 80, 145, 144, 97, 0, 156, 86, 91, 96, 64, 81, 128, 145, 3, 144, 243, 91, 97, 0, 97, 97, 0, 105, 86, 91, 0, 91, 96, 0, 84, 129, 86, 91, 91, 96, 1, 21, 97, 0, 139, 87, 96, 1, 96, 0, 84, 97, 0, 128, 145, 144, 97, 0, 183, 86, 91, 96, 0, 129, 144, 85, 80, 97, 0, 106, 86, 91, 86, 91, 97, 0, 150, 129, 97, 1, 13, 86, 91, 130, 82, 80, 80, 86, 91, 96, 0, 96, 32, 130, 1, 144, 80, 97, 0, 177, 96, 0, 131, 1, 132, 97, 0, 141, 86, 91, 146, 145, 80, 80, 86, 91, 96, 0, 97, 0, 194, 130, 97, 1, 13, 86, 91, 145, 80, 97, 0, 205, 131, 97, 1, 13, 86, 91, 146, 80, 130, 127, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 3, 130, 17, 21, 97, 1, 2, 87, 97, 1, 1, 97, 1, 23, 86, 91, 91, 130, 130, 1, 144, 80, 146, 145, 80, 80, 86, 91, 96, 0, 129, 144, 80, 145, 144, 80, 86, 91, 127, 78, 72, 123, 113, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 82, 96, 17, 96, 4, 82, 96, 36, 96, 0, 253, 254, 162, 100, 105, 112, 102, 115, 88, 34, 18, 32, 188, 171, 3, 133, 22, 125, 190, 210, 141, 238, 52, 241, 197, 179, 14, 207, 205, 103, 145, 84, 149, 240, 163, 45, 46, 234, 218, 142, 9, 65, 147, 163, 100, 115, 111, 108, 99, 67, 0, 8, 3, 0, 51];

		let caller = H160::default();

		let mut nonce: u64 = 0;
		let nonce_as_u256: U256 = nonce.into();

		let value = U256::default();
		let gas_limit_create: u64 = 1_250_000 * 1_000_000_000;
		let create_runner_results = T::Runner::create(
			caller,
			contract_bytecode,
			value,
			gas_limit_create,
			None,
			None,
			Some(nonce_as_u256),
			vec![],
			T::config(),
		);
		assert_eq!(create_runner_results.is_ok(), true, "create() failed");

		// derive the resulting contract address from our create
		let mut rlp = RlpStream::new_list(2);
		rlp.append(&caller);
		rlp.append(&0u8);
		let contract_address = H160::from_slice(&Keccak256::digest(&rlp.out())[12..]);

		// derive encoded contract call -- in this case, just the function selector
		let mut encoded_call = vec![0u8; 4];
		encoded_call[0..4].copy_from_slice(&Keccak256::digest(b"infinite()")[0..4]);

		let gas_limit_call = x as u64;

	}: {

		nonce = nonce + 1;
		let nonce_as_u256: U256 = nonce.into();

		let call_runner_results = T::Runner::call(
			caller,
			contract_address,
			encoded_call,
			value,
			gas_limit_call,
			None,
			None,
			Some(nonce_as_u256),
			vec![],
			T::config(),
		);
		assert_eq!(call_runner_results.is_ok(), true, "call() failed");
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use frame_support::assert_ok;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}

	#[test]
	fn test_runner_execute() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_runner_execute::<Test>());
		});
	}
}
