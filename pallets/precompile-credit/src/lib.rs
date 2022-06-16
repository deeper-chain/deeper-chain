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

mod util;

use codec::Decode;
use core::marker::PhantomData;
use fp_evm::{
    Context, ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileOutput,
    PrecompileResult,
};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_credit::Call as CreditCall;
use pallet_credit::CreditInterface;
use pallet_evm::AddressMapping;

use crate::util::{Gasometer, RuntimeHelper};
use alloc::{borrow::ToOwned, vec};
use arrayref::array_ref;
use sp_core::{H160, U256};

const BASIC_LEN: usize = 4 + 32;
const SELECTOR_GET_CREDIT_SCORE: u32 = 0x87135d7d;
const SELECTOR_ADD_CREDIT_SCORE: u32 = 0x5915ad98;
const SELECTOR_SLASH_CREDIT_SCORE: u32 = 0xa62184b3;

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
        selector:
        0x5915ad98: add_credit_score
        0xa62184b3: slash_credit_score
        0x87135d7d: get_credit_score
        */

        if input.len() < BASIC_LEN {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("input len not enough".into()),
            });
        }
        let mut gasometer = Gasometer::new(target_gas);
        let gasometer = &mut gasometer;

        let real_type = u32::from_be_bytes(array_ref!(input, 0, 4).to_owned());
        match real_type {
            SELECTOR_GET_CREDIT_SCORE => {
                Self::get_credit_score(input, context, is_static, gasometer)
            }
            SELECTOR_ADD_CREDIT_SCORE => {
                Self::add_credit_score(input, context, is_static, gasometer)
            }
            SELECTOR_SLASH_CREDIT_SCORE => {
                Self::slash_credit_score(input, context, is_static, gasometer)
            }
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
    pub fn get_credit_score(
        input: &[u8],
        context: &Context,
        is_static: bool,
        gasometer: &mut Gasometer,
    ) -> PrecompileResult {
        Self::check_input_len(input, BASIC_LEN)?;

        gasometer.check_function_modifier(context, is_static, util::FunctionModifier::View)?;

        let account = H160::from(array_ref!(input, BASIC_LEN - 20, 20));
        let origin = Runtime::AddressMapping::into_account_id(account);

        let score = pallet_credit::Pallet::<Runtime>::get_credit_score(&origin);
        let score = U256::from(score.unwrap_or(0));
        let mut output = vec![0; 32];
        score.to_big_endian(&mut output);
        gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost() * 2)?;

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gasometer.used_gas(),
            output,
            logs: Default::default(),
        })
    }

    pub fn add_credit_score(
        input: &[u8],
        context: &Context,
        is_static: bool,
        gasometer: &mut Gasometer,
    ) -> PrecompileResult {
        Self::do_update_credit(input, context, is_static, gasometer, true)?;

        let weight = RuntimeHelper::<Runtime>::db_read_gas_cost() * 2
            + RuntimeHelper::<Runtime>::db_write_gas_cost();
        gasometer.record_cost(weight)?;
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gasometer.used_gas(),
            output: Default::default(),
            logs: Default::default(),
        })
    }

    pub fn slash_credit_score(
        input: &[u8],
        context: &Context,
        is_static: bool,
        gasometer: &mut Gasometer,
    ) -> PrecompileResult {
        Self::do_update_credit(input, context, is_static, gasometer, false)?;
        let weight = RuntimeHelper::<Runtime>::db_read_gas_cost() * 4
            + RuntimeHelper::<Runtime>::db_write_gas_cost() * 2;
        gasometer.record_cost(weight)?;

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gasometer.used_gas(),
            output: Default::default(),
            logs: Default::default(),
        })
    }

    fn do_update_credit(
        input: &[u8],
        context: &Context,
        is_static: bool,
        gasometer: &mut Gasometer,
        add_flag: bool,
    ) -> Result<(), PrecompileFailure> {
        Self::check_input_len(input, BASIC_LEN + 32)?;
        gasometer.check_function_modifier(
            context,
            is_static,
            util::FunctionModifier::NonPayable,
        )?;
        let account = H160::from(array_ref!(input, BASIC_LEN - 20, 20));
        let score = U256::from(array_ref!(input, BASIC_LEN, 32)).low_u64();
        let origin = Runtime::AddressMapping::into_account_id(account);

        pallet_credit::Pallet::<Runtime>::evm_update_credit(
            &context.caller,
            &origin,
            score,
            add_flag,
        );
        Ok(())
    }

    fn check_input_len(input: &[u8], base_len: usize) -> Result<(), PrecompileFailure> {
        if input.len() < base_len {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("input len not enough".into()),
            });
        }
        Ok(())
    }
}
