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

//! # EVM Pallet
//!
//! The EVM pallet allows unmodified EVM code to be executed in a Substrate-based blockchain.
//! - [`evm::Config`]
//!
//! ## EVM Engine
//!
//! The EVM pallet uses [`SputnikVM`](https://github.com/rust-blockchain/evm) as the underlying EVM engine.
//! The engine is overhauled so that it's [`modular`](https://github.com/corepaper/evm).
//!
//! ## Execution Lifecycle
//!
//! There are a separate set of accounts managed by the EVM pallet. Substrate based accounts can call the EVM Pallet
//! to deposit or withdraw balance from the Substrate base-currency into a different balance managed and used by
//! the EVM pallet. Once a user has populated their balance, they can create and call smart contracts using this pallet.
//!
//! There's one-to-one mapping from Substrate accounts and EVM external accounts that is defined by a conversion function.
//!
//! ## EVM Pallet vs Ethereum Network
//!
//! The EVM pallet should be able to produce nearly identical results compared to the Ethereum mainnet,
//! including gas cost and balance changes.
//!
//! Observable differences include:
//!
//! - The available length of block hashes may not be 256 depending on the configuration of the System pallet
//! in the Substrate runtime.
//! - Difficulty and coinbase, which do not make sense in this pallet and is currently hard coded to zero.
//!
//! We currently do not aim to make unobservable behaviors, such as state root, to be the same. We also don't aim to follow
//! the exact same transaction / receipt format. However, given one Ethereum transaction and one Substrate account's
//! private key, one should be able to convert any Ethereum transaction into a transaction compatible with this pallet.
//!
//! The gas configurations are configurable. Right now, a pre-defined London hard fork configuration option is provided.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
pub mod runner;
#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod benchmarks;

pub use crate::runner::Runner;
pub use evm::{Context, ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed};
pub use fp_evm::{
	Account, CallInfo, CreateInfo, ExecutionInfo, LinearCostPrecompile, Log, Precompile,
	PrecompileFailure, PrecompileOutput, PrecompileResult, PrecompileSet, Vicinity,
};

#[cfg(feature = "std")]
use codec::{Decode, Encode};
use evm::Config as EvmConfig;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	traits::{
		Currency, ExistenceRequirement, FindAuthor, Get, Imbalance, IsType, OnUnbalanced,
		SignedImbalance, WithdrawReasons,
	},
	weights::{Pays, PostDispatchInfo, Weight},
};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{ecdsa, H160, H256, U256};
use sp_io::{crypto::secp256k1_ecdsa_recover, hashing::keccak_256};
use sp_runtime::{
	traits::{Saturating, UniqueSaturatedInto, Zero},
	AccountId32, DispatchError,
};
use sp_std::vec::Vec;

pub type EcdsaSignature = ecdsa::Signature;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Calculator for current gas price.
		type FeeCalculator: FeeCalculator;

		/// Maps Ethereum gas to Substrate weight.
		type GasWeightMapping: GasWeightMapping;

		/// Block number to block hash.
		type BlockHashMapping: BlockHashMapping;

		/// Mapping from address to account id.
		type AddressMapping: AddressMapping<Self::AccountId>;
		/// Currency type for withdraw and balance storage.
		type Currency: Currency<Self::AccountId>;

		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Precompiles associated with this EVM engine.
		type PrecompilesType: PrecompileSet;
		type PrecompilesValue: Get<Self::PrecompilesType>;
		/// Chain ID of EVM.
		type ChainId: Get<u64>;
		/// The block gas limit. Can be a simple constant, or an adjustment algorithm in another pallet.
		type BlockGasLimit: Get<U256>;
		/// EVM execution runner.
		type Runner: Runner<Self>;

		/// To handle fee deduction for EVM transactions. An example is this pallet being used by `pallet_ethereum`
		/// where the chain implementing `pallet_ethereum` should be able to configure what happens to the fees
		/// Similar to `OnChargeTransaction` of `pallet_transaction_payment`
		type OnChargeTransaction: OnChargeEVMTransaction<Self>;

		/// Find author for the current block.
		type FindAuthor: FindAuthor<H160>;

		/// EVM config used in the module.
		fn config() -> &'static EvmConfig {
			&LONDON_CONFIG
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// set up a Substrate account and Eth account one-to-one mapping.
		/// - `eth_address`: The Eth address to bind to the caller's Substrate account
		/// - `eth_signature`: A signature to prove the ownership Eth address
		// todo: 1.weight, 2.cancel account pair
		#[pallet::weight(0)]
		pub fn pair_accounts(
			origin: OriginFor<T>,
			eth_address: H160,
			eth_signature: EcdsaSignature,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// ensure account_id and eth_address have NOT been mapped
			ensure!(
				!EthAddresses::<T>::contains_key(&who),
				Error::<T>::AccountIdHasMapped
			);
			ensure!(
				!Accounts::<T>::contains_key(eth_address),
				Error::<T>::EthAddressHasMapped
			);

			// recover evm address from signature
			let address =
				Self::eth_recover(&eth_signature, &who.using_encoded(to_ascii_hex), &[][..])
					.ok_or(Error::<T>::BadSignature)?;
			ensure!(eth_address == address, Error::<T>::InvalidSignature);

			// check if the evm padded address already exists
			let account_id = T::AddressMapping::into_account_id(eth_address);
			if frame_system::Pallet::<T>::account_exists(&account_id) {
				let free_balance = T::Currency::free_balance(&account_id);
				T::Currency::transfer(
					&account_id,
					&who,
					free_balance,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			Accounts::<T>::insert(eth_address, &who);
			EthAddresses::<T>::insert(&who, address);

			Self::deposit_event(Event::PairedAccounts(who, eth_address));
			Ok(().into())
		}

		/// Issue an EVM call operation. This is similar to a message call transaction in Ethereum.
		#[pallet::weight(T::GasWeightMapping::gas_to_weight(*gas_limit))]
		pub fn call(
			origin: OriginFor<T>,
			source: H160,
			target: H160,
			input: Vec<u8>,
			value: U256,
			gas_limit: u64,
			max_fee_per_gas: U256,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			T::AddressMapping::ensure_address_origin(&source, &who)?;

			let info = T::Runner::call(
				source,
				target,
				input,
				value,
				gas_limit,
				Some(max_fee_per_gas),
				max_priority_fee_per_gas,
				nonce,
				access_list,
				T::config(),
			)?;

			match info.exit_reason {
				ExitReason::Succeed(_) => {
					Pallet::<T>::deposit_event(Event::<T>::Executed(target));
				}
				_ => {
					Pallet::<T>::deposit_event(Event::<T>::ExecutedFailed(target));
				}
			};

			Ok(PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(
					info.used_gas.unique_saturated_into(),
				)),
				pays_fee: Pays::No,
			})
		}

		/// Issue an EVM create operation. This is similar to a contract creation transaction in
		/// Ethereum.
		#[pallet::weight(T::GasWeightMapping::gas_to_weight(*gas_limit))]
		pub fn create(
			origin: OriginFor<T>,
			source: H160,
			init: Vec<u8>,
			value: U256,
			gas_limit: u64,
			max_fee_per_gas: U256,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			T::AddressMapping::ensure_address_origin(&source, &who)?;

			let info = T::Runner::create(
				source,
				init,
				value,
				gas_limit,
				Some(max_fee_per_gas),
				max_priority_fee_per_gas,
				nonce,
				access_list,
				T::config(),
			)?;

			match info {
				CreateInfo {
					exit_reason: ExitReason::Succeed(_),
					value: create_address,
					..
				} => {
					Pallet::<T>::deposit_event(Event::<T>::Created(create_address));
				}
				CreateInfo {
					exit_reason: _,
					value: create_address,
					..
				} => {
					Pallet::<T>::deposit_event(Event::<T>::CreatedFailed(create_address));
				}
			}

			Ok(PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(
					info.used_gas.unique_saturated_into(),
				)),
				pays_fee: Pays::No,
			})
		}

		/// Issue an EVM create2 operation.
		#[pallet::weight(T::GasWeightMapping::gas_to_weight(*gas_limit))]
		pub fn create2(
			origin: OriginFor<T>,
			source: H160,
			init: Vec<u8>,
			salt: H256,
			value: U256,
			gas_limit: u64,
			max_fee_per_gas: U256,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			access_list: Vec<(H160, Vec<H256>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			T::AddressMapping::ensure_address_origin(&source, &who)?;

			let info = T::Runner::create2(
				source,
				init,
				salt,
				value,
				gas_limit,
				Some(max_fee_per_gas),
				max_priority_fee_per_gas,
				nonce,
				access_list,
				T::config(),
			)?;

			match info {
				CreateInfo {
					exit_reason: ExitReason::Succeed(_),
					value: create_address,
					..
				} => {
					Pallet::<T>::deposit_event(Event::<T>::Created(create_address));
				}
				CreateInfo {
					exit_reason: _,
					value: create_address,
					..
				} => {
					Pallet::<T>::deposit_event(Event::<T>::CreatedFailed(create_address));
				}
			}

			Ok(PostDispatchInfo {
				actual_weight: Some(T::GasWeightMapping::gas_to_weight(
					info.used_gas.unique_saturated_into(),
				)),
				pays_fee: Pays::No,
			})
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Ethereum events from contracts.
		Log(Log),
		/// A contract has been created at given \[address\].
		Created(H160),
		/// A \[contract\] was attempted to be created, but the execution failed.
		CreatedFailed(H160),
		/// A \[contract\] has been executed successfully with states applied.
		Executed(H160),
		/// A \[contract\] has been executed with errors. States are reverted with only gas fees applied.
		ExecutedFailed(H160),
		/// A deposit has been made at a given address. \[sender, address, value\]
		BalanceDeposit(T::AccountId, H160, U256),
		/// A withdrawal has been made from a given address. \[sender, address, value\]
		BalanceWithdraw(T::AccountId, H160, U256),
		/// Mapping between Substrate accounts and Eth accounts
		PairedAccounts(T::AccountId, H160),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough balance to perform action
		BalanceLow,
		/// Calculating total fee overflowed
		FeeOverflow,
		/// Calculating total payment overflowed
		PaymentOverflow,
		/// Withdraw fee failed
		WithdrawFailed,
		/// Gas price is too low.
		GasPriceTooLow,
		/// Nonce is invalid
		InvalidNonce,
		/// AccountId has mapped
		AccountIdHasMapped,
		/// Eth address has mapped
		EthAddressHasMapped,
		/// Bad signature
		BadSignature,
		/// Invalid signature
		InvalidSignature,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub account_pairs: std::collections::BTreeMap<H160, T::AccountId>,
		pub accounts: std::collections::BTreeMap<H160, GenesisAccount>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				account_pairs: Default::default(),
				accounts: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (eth_addr, account_id) in &self.account_pairs {
				<Accounts<T>>::insert(eth_addr, account_id);
				<EthAddresses<T>>::insert(account_id, eth_addr);
			}
			for (address, account) in &self.accounts {
				let account_id = T::AddressMapping::into_account_id(*address);

				// ASSUME: in one single EVM transaction, the nonce will not increase more than
				// `u128::max_value()`.
				for _ in 0..account.nonce.low_u128() {
					frame_system::Pallet::<T>::inc_account_nonce(&account_id);
				}

				T::Currency::deposit_creating(
					&account_id,
					account.balance.low_u128().unique_saturated_into(),
				);

				Pallet::<T>::create_account(*address, account.code.clone());

				for (index, value) in &account.storage {
					<AccountStorages<T>>::insert(address, index, value);
				}
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_codes)]
	pub type AccountCodes<T: Config> = StorageMap<_, Blake2_128Concat, H160, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_storages)]
	pub type AccountStorages<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, H160, Blake2_128Concat, H256, H256, ValueQuery>;

	/// Eth Address => AccountId
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, H160, T::AccountId, ValueQuery>;

	/// AccountId => Eth Address
	#[pallet::storage]
	#[pallet::getter(fn eth_addresses)]
	pub type EthAddresses<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, H160, ValueQuery>;
}

/// Type alias for currency balance.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Type alias for negative imbalance during fees
type NegativeImbalanceOf<C, T> =
	<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

/// Trait that outputs the current transaction gas price.
pub trait FeeCalculator {
	/// Return the minimal required gas price.
	fn min_gas_price() -> U256;
}

impl FeeCalculator for () {
	fn min_gas_price() -> U256 {
		U256::zero()
	}
}

pub trait AddressMapping<AccountId> {
	fn into_account_id(address: H160) -> AccountId;
	fn ensure_address_origin(address: &H160, origin: &AccountId) -> Result<(), DispatchError>;
}

pub struct PairedAddressMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AddressMapping<T::AccountId> for PairedAddressMapping<T>
where
	T::AccountId: IsType<AccountId32>,
{
	fn into_account_id(address: H160) -> T::AccountId {
		if Accounts::<T>::contains_key(&address) {
			Accounts::<T>::get(address)
		} else {
			let mut data: [u8; 32] = [0u8; 32];
			data[0..4].copy_from_slice(b"evm:");
			data[4..24].copy_from_slice(&address[..]);
			AccountId32::from(data).into()
		}
	}

	fn ensure_address_origin(address: &H160, origin: &T::AccountId) -> Result<(), DispatchError> {
		if Accounts::<T>::contains_key(&address) && Accounts::<T>::get(address) == *origin {
			Ok(())
		} else {
			Err(DispatchError::Other(
				"eth and substrate addresses are not paired",
			))
		}
	}
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice
/// the length.
pub fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
	let mut r = Vec::with_capacity(data.len() * 2);
	let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
	for &b in data.iter() {
		push_nibble(b / 16);
		push_nibble(b % 16);
	}
	r
}

/// A trait for getting a block hash by number.
pub trait BlockHashMapping {
	fn block_hash(number: u32) -> H256;
}

/// Returns the Substrate block hash by number.
pub struct SubstrateBlockHashMapping<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> BlockHashMapping for SubstrateBlockHashMapping<T> {
	fn block_hash(number: u32) -> H256 {
		let number = T::BlockNumber::from(number);
		H256::from_slice(frame_system::Pallet::<T>::block_hash(number).as_ref())
	}
}

/// A mapping function that converts Ethereum gas to Substrate weight
pub trait GasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight;
	fn weight_to_gas(weight: Weight) -> u64;
}

impl GasWeightMapping for () {
	fn gas_to_weight(gas: u64) -> Weight {
		gas as Weight
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		weight as u64
	}
}

static LONDON_CONFIG: EvmConfig = EvmConfig::london();

#[cfg(feature = "std")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, Serialize, Deserialize)]
/// Account definition used for genesis block construction.
pub struct GenesisAccount {
	/// Account nonce.
	pub nonce: U256,
	/// Account balance.
	pub balance: U256,
	/// Full account storage.
	pub storage: std::collections::BTreeMap<H256, H256>,
	/// Account code.
	pub code: Vec<u8>,
}

impl<T: Config> Pallet<T> {
	// Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign`
	// would sign.
	pub fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
		let prefix = b"deeper evm:";
		let mut l = prefix.len() + what.len() + extra.len();
		let mut rev = Vec::new();
		while l > 0 {
			rev.push(b'0' + (l % 10) as u8);
			l /= 10;
		}
		let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
		v.extend(rev.into_iter().rev());
		v.extend_from_slice(&prefix[..]);
		v.extend_from_slice(what);
		v.extend_from_slice(extra);
		v
	}

	// Attempts to recover the Ethereum address from a message signature signed by
	// using the Ethereum RPC's `personal_sign` and `eth_sign`.
	pub fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<H160> {
		let msg = keccak_256(&Self::ethereum_signable_message(what, extra));
		let mut res = H160::default();
		res.0
			.copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
		Some(res)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
	// Returns an Etherum public key derived from an Ethereum secret key.
	pub fn eth_public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
		libsecp256k1::PublicKey::from_secret_key(secret)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
	// Constructs a message and signs it.
	pub fn eth_sign(secret: &libsecp256k1::SecretKey, what: &[u8], extra: &[u8]) -> EcdsaSignature {
		let msg = keccak_256(&Self::ethereum_signable_message(
			&to_ascii_hex(what)[..],
			extra,
		));
		let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
		let mut r = [0u8; 65];
		r[0..64].copy_from_slice(&sig.serialize()[..]);
		r[64] = recovery_id.serialize();
		EcdsaSignature::from_slice(&r)
	}

	/// Check whether an account is empty.
	pub fn is_account_empty(address: &H160) -> bool {
		let account = Self::account_basic(address);
		let code_len = <AccountCodes<T>>::decode_len(address).unwrap_or(0);

		account.nonce == U256::zero() && account.balance == U256::zero() && code_len == 0
	}

	/// Remove an account if its empty.
	pub fn remove_account_if_empty(address: &H160) {
		if Self::is_account_empty(address) {
			Self::remove_account(address);
		}
	}

	/// Remove an account.
	pub fn remove_account(address: &H160) {
		if <AccountCodes<T>>::contains_key(address) {
			let account_id = T::AddressMapping::into_account_id(*address);
			let _ = frame_system::Pallet::<T>::dec_sufficients(&account_id);
		}

		<AccountCodes<T>>::remove(address);
		<AccountStorages<T>>::remove_prefix(address, None);
	}

	/// Create an account.
	pub fn create_account(address: H160, code: Vec<u8>) {
		if code.is_empty() {
			return;
		}

		if !<AccountCodes<T>>::contains_key(&address) {
			let account_id = T::AddressMapping::into_account_id(address);
			let _ = frame_system::Pallet::<T>::inc_sufficients(&account_id);
		}

		<AccountCodes<T>>::insert(address, code);
	}

	/// Get the account basic in EVM format.
	pub fn account_basic(address: &H160) -> Account {
		let account_id = T::AddressMapping::into_account_id(*address);

		let nonce = frame_system::Pallet::<T>::account_nonce(&account_id);
		// keepalive `true` takes into account ExistentialDeposit as part of what's considered liquid balance.
		let balance = T::Currency::free_balance(&account_id);

		Account {
			nonce: U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(nonce)),
			balance: U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(balance)),
		}
	}

	/// Get the author using the FindAuthor trait.
	pub fn find_author() -> H160 {
		let digest = <frame_system::Pallet<T>>::digest();
		let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());

		T::FindAuthor::find_author(pre_runtime_digests).unwrap_or_default()
	}
}

/// Handle withdrawing, refunding and depositing of transaction fees.
/// Similar to `OnChargeTransaction` of `pallet_transaction_payment`
pub trait OnChargeEVMTransaction<T: Config> {
	type LiquidityInfo: Default;

	/// Before the transaction is executed the payment of the transaction fees
	/// need to be secured.
	fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, Error<T>>;

	/// After the transaction was executed the actual fee can be calculated.
	/// This function should refund any overpaid fees and optionally deposit
	/// the corrected amount.
	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		priority_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	);

	/// Introduced in EIP1559 to handle the priority tip payment to the block Author.
	fn pay_priority_fee(tip: U256);
}

/// Implements the transaction payment for a pallet implementing the `Currency`
/// trait (eg. the pallet_balances) using an unbalance handler (implementing
/// `OnUnbalanced`).
/// Similar to `CurrencyAdapter` of `pallet_transaction_payment`
pub struct EVMCurrencyAdapter<C, OU>(sp_std::marker::PhantomData<(C, OU)>);

impl<T, C, OU> OnChargeEVMTransaction<T> for EVMCurrencyAdapter<C, OU>
where
	T: Config,
	C: Currency<<T as frame_system::Config>::AccountId>,
	C::PositiveImbalance: Imbalance<
		<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnUnbalanced<NegativeImbalanceOf<C, T>>,
{
	// Kept type as Option to satisfy bound of Default
	type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;

	fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, Error<T>> {
		let account_id = T::AddressMapping::into_account_id(*who);
		let imbalance = C::withdraw(
			&account_id,
			fee.low_u128().unique_saturated_into(),
			WithdrawReasons::FEE,
			ExistenceRequirement::AllowDeath,
		)
		.map_err(|_| Error::<T>::BalanceLow)?;
		Ok(Some(imbalance))
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		priority_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) {
		if let Some(paid) = already_withdrawn {
			let account_id = T::AddressMapping::into_account_id(*who);

			// Calculate how much refund we should return
			let refund_amount = paid
				.peek()
				.saturating_sub(corrected_fee.low_u128().unique_saturated_into());
			// refund to the account that paid the fees. If this fails, the
			// account might have dropped below the existential balance. In
			// that case we don't refund anything.
			let refund_imbalance = C::deposit_into_existing(&account_id, refund_amount)
				.unwrap_or_else(|_| C::PositiveImbalance::zero());

			// Make sure this works with 0 ExistentialDeposit
			// https://github.com/paritytech/substrate/issues/10117
			// If we tried to refund something, the account still empty and the ED is set to 0,
			// we call `make_free_balance_be` with the refunded amount.
			let refund_imbalance = if C::minimum_balance().is_zero()
				&& refund_amount > C::Balance::zero()
				&& C::total_balance(&account_id).is_zero()
			{
				// Known bug: Substrate tried to refund to a zeroed AccountData, but
				// interpreted the account to not exist.
				match C::make_free_balance_be(&account_id, refund_amount) {
					SignedImbalance::Positive(p) => p,
					_ => C::PositiveImbalance::zero(),
				}
			} else {
				refund_imbalance
			};

			let mut adjusted_paid = paid
				.offset(refund_imbalance)
				.same()
				.unwrap_or_else(|_| C::NegativeImbalance::zero());
			if adjusted_paid.peek() > priority_fee.low_u128().unique_saturated_into() {
				adjusted_paid = adjusted_paid.split(priority_fee.low_u128().unique_saturated_into()).1;
				OU::on_unbalanced(adjusted_paid);
			}
		}
	}

	fn pay_priority_fee(tip: U256) {
		let account_id = T::AddressMapping::into_account_id(<Pallet<T>>::find_author());
		let _ = C::deposit_into_existing(&account_id, tip.low_u128().unique_saturated_into());
	}
}

/// Implementation for () does not specify what to do with imbalance
impl<T> OnChargeEVMTransaction<T> for ()
	where
	T: Config,
	<T::Currency as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance:
		Imbalance<<T::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance, Opposite = <T::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance>,
	<T::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance:
		Imbalance<<T::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance, Opposite = <T::Currency as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance>, {
	// Kept type as Option to satisfy bound of Default
	type LiquidityInfo = Option<NegativeImbalanceOf<T::Currency, T>>;

	fn withdraw_fee(
		who: &H160,
		fee: U256,
	) -> Result<Self::LiquidityInfo, Error<T>> {
		EVMCurrencyAdapter::<<T as Config>::Currency, ()>::withdraw_fee(who, fee)
	}

	fn correct_and_deposit_fee(
		who: &H160,
		corrected_fee: U256,
		priority_fee: U256,
		already_withdrawn: Self::LiquidityInfo,
	) {
		<EVMCurrencyAdapter::<<T as Config>::Currency, ()> as OnChargeEVMTransaction<T>>::correct_and_deposit_fee(who, corrected_fee, priority_fee, already_withdrawn)
	}

	fn pay_priority_fee(tip: U256) {
		<EVMCurrencyAdapter::<<T as Config>::Currency, ()> as OnChargeEVMTransaction<T>>::pay_priority_fee(tip);
	}
}
