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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use codec::Decode;
use core::marker::PhantomData;
use fp_evm::{
    Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
    PrecompileResult,
};
use frame_support::{
    dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
    weights::{DispatchClass, Pays},
};
use pallet_credit::CreditInterface;
use pallet_evm::{AddressMapping, GasWeightMapping};
use pallet_credit::Call as CreditCall;

use sp_core::{H160, H256, U256};
use alloc::{vec,vec::Vec,borrow::ToOwned};
use arrayref::array_ref;

const BASIC_LEN: usize = 4 + 32;

// from moonbeam
/// Represents modifiers a Solidity function can be annotated with.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionModifier {
	/// Function that doesn't modify the state.
	View,
	/// Function that modifies the state but refuse receiving funds.
	/// Correspond to a Solidity function with no modifiers.
	NonPayable,
	/// Function that modifies the state and accept funds.
	Payable,
}

fn check_function_modifier(
    context: &Context,
    is_static: bool,
    modifier: FunctionModifier,
) ->  Result<(),PrecompileFailure> {
    if is_static && modifier != FunctionModifier::View {
        return Err(PrecompileFailure::Error {
            exit_status: ExitError::Other("can't call non-static function in static context".into()),
        });
    }

    if modifier != FunctionModifier::Payable && context.apparent_value > U256::zero() {
        return Err(PrecompileFailure::Error {
            exit_status: ExitError::Other("function is not payable".into()),
        });
    }

    Ok(())
}

pub struct CreditDispatch<Runtime> {
    _marker: PhantomData<Runtime>,
}


impl<Runtime> Precompile for CreditDispatch<Runtime>
where
    Runtime: pallet_credit::Config + pallet_evm::Config,
    Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
    <Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
    Runtime::Call: From<CreditCall<Runtime>>,
{
    fn execute(
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
        is_static: bool,
    ) -> PrecompileResult {
        /*
		selecter:
        0x5915ad98: add_credit_score
        0xa62184b3: slash_credit_score
        0x87135d7d: get_credit_score
        */

        if input.len() < BASIC_LEN {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("input len not enough".into()),
            });
        }
        let real_type = u32::from_be_bytes(array_ref!(input, 0, 4).to_owned());
        match real_type {
            0x87135d7d => Self::get_credit_score(input, context,is_static),
            0x5915ad98 => Self::add_credit_score(input, context,is_static),
            0xa62184b3 => Self::slash_credit_score(input, context,is_static),
            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidCode,
            }),
        }
    }
}

impl<Runtime> CreditDispatch<Runtime>
where
    Runtime: pallet_credit::Config + pallet_evm::Config,
    Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
    <Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
    Runtime::Call: From<CreditCall<Runtime>>,
{
    pub fn get_credit_score(input: &[u8], context: &Context, is_static:bool) -> PrecompileResult {
		Self::check_input_len(input, BASIC_LEN)?;
        check_function_modifier(context,is_static, FunctionModifier::View)?;
        let account = H160::from(array_ref!(input, BASIC_LEN - 20, 20));
        let origin = Runtime::AddressMapping::into_account_id(account);
        let score = pallet_credit::Pallet::<Runtime>::get_credit_score(&origin);
        let score = U256::from(score.unwrap_or(0));
        let mut output = vec![0; 32];
        score.to_big_endian(&mut output);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: 2000,
            output,
            logs: Default::default(),
        })
    }

    pub fn add_credit_score(input: &[u8], context: &Context, is_static:bool) -> PrecompileResult {
		Self::check_input_len(input, BASIC_LEN + 32)?;
        check_function_modifier(context,is_static, FunctionModifier::NonPayable)?;
        let account = H160::from(array_ref!(input, BASIC_LEN - 20, 20));
        let score = U256::from(array_ref!(input, BASIC_LEN, 32)).low_u64();
        let origin = Runtime::AddressMapping::into_account_id(account);

        pallet_credit::Pallet::<Runtime>::add_or_update_credit(origin, score);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: 2000,
            output: Default::default(),
            logs: Default::default(),
        })
    }

    pub fn slash_credit_score(input: &[u8], context: &Context, is_static:bool) -> PrecompileResult {
       
		Self::check_input_len(input, BASIC_LEN + 32)?;
        check_function_modifier(context,is_static, FunctionModifier::NonPayable)?;
        let account = H160::from(array_ref!(input, BASIC_LEN - 20, 20));
        let score = U256::from(array_ref!(input, BASIC_LEN, 32)).low_u64();
        let origin = Runtime::AddressMapping::into_account_id(account);

        pallet_credit::Pallet::<Runtime>::slash_credit(&origin, Some(score));

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: 2000,
            output: Default::default(),
            logs: Default::default(),
        })
    }

	fn check_input_len(input: &[u8],base_len :usize) -> Result<(),PrecompileFailure> {
		if input.len() < base_len {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("input len not enough".into()),
            });
        }
		Ok(())
	}
}
