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

//! Test utilities

use super::*;
use crate::IntermediateStateRoot;
use ethereum::{TransactionAction, TransactionSignature};
use frame_support::{
    parameter_types,
    traits::{FindAuthor, GenesisBuild, IsType},
    ConsensusEngineId, PalletId,
};
use pallet_evm::{AddressMapping, FeeCalculator};
use rlp::*;
use sha3::Digest;
use sp_core::{H160, H256, U256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};
use std::{collections::BTreeMap, str::FromStr};

pub type SignedExtra = (frame_system::CheckSpecVersion<Test>,);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test, (), SignedExtra>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime! {
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        EVM: pallet_evm::{Pallet, Call, Storage, Event<T>},
        Ethereum: crate::{Pallet, Call, Storage, Event, Origin},
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Call = Call;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
    pub const ExistentialDeposit: u64 = 500;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> U256 {
        1.into()
    }
}

pub struct FindAuthorTruncated;
impl FindAuthor<H160> for FindAuthorTruncated {
    fn find_author<'a, I>(_digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        Some(address_build(0).address)
    }
}

parameter_types! {
    pub const TransactionByteFee: u64 = 1;
    pub const ChainId: u64 = 42;
    pub const EVMModuleId: PalletId = PalletId(*b"py/evmpa");
    pub const BlockGasLimit: U256 = U256::MAX;
}

pub struct PairedAddressMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AddressMapping<T::AccountId> for PairedAddressMapping<T>
where
    T::AccountId: IsType<AccountId32>,
{
    // Returns the AccountId used go generate the given Eth Address.
    fn into_account_id(address: H160) -> T::AccountId {
        if pallet_evm::Accounts::<T>::contains_key(&address) {
            pallet_evm::Accounts::<T>::get(address)
        } else {
            let mut data: [u8; 32] = [0u8; 32];
            data[0..4].copy_from_slice(b"evm:");
            data[4..24].copy_from_slice(&address[..]);
            AccountId32::from(data).into()
        }
    }

    fn ensure_address_origin(address: &H160, origin: &T::AccountId) -> Result<(), DispatchError> {
        if pallet_evm::Accounts::<T>::contains_key(&address)
            && pallet_evm::Accounts::<T>::get(address) == *origin
        {
            Ok(())
        } else {
            Err(DispatchError::Other(
                "eth and substrate addresses are not paired",
            ))
        }
    }
}

impl pallet_evm::Config for Test {
    type FeeCalculator = FixedGasPrice;
    type GasWeightMapping = ();
    type AddressMapping = PairedAddressMapping<Test>;
    type Currency = Balances;
    type Event = Event;
    type PrecompilesType = ();
    type PrecompilesValue = ();
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type ChainId = ChainId;
    type BlockGasLimit = BlockGasLimit;
    type OnChargeTransaction = ();
    type FindAuthor = FindAuthorTruncated;
    type BlockHashMapping = crate::EthereumBlockHashMapping<Self>;
}

impl crate::Config for Test {
    type Event = Event;
    type StateRoot = IntermediateStateRoot;
}

impl fp_self_contained::SelfContainedCall for Call {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            Call::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(&self, info: &Self::SignedInfo) -> Option<TransactionValidity> {
        match self {
            Call::Ethereum(call) => call.validate_self_contained(info),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.pre_dispatch_self_contained(info),
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>>
    {
        use sp_runtime::traits::Dispatchable as _;
        match self {
            call @ Call::Ethereum(crate::Call::transact { .. }) => {
                Some(call.dispatch(Origin::from(crate::RawOrigin::EthereumTransaction(info))))
            }
            _ => None,
        }
    }
}

pub struct AccountInfo {
    pub address: H160,
    pub account_id: AccountId32,
    pub private_key: H256,
}

fn address_build(seed: u8) -> AccountInfo {
    let private_key = H256::from_slice(&[(seed + 1) as u8; 32]); //H256::from_low_u64_be((i + 1) as u64);
    let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
    let public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
    let address = H160::from(H256::from_slice(&Keccak256::digest(public_key)[..]));

    let mut data = [0u8; 32];
    data[0..20].copy_from_slice(&address[..]);

    AccountInfo {
        private_key,
        account_id: AccountId32::from(Into::<[u8; 32]>::into(data)),
        address,
    }
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext(accounts_len: usize) -> (Vec<AccountInfo>, sp_io::TestExternalities) {
    // sc_cli::init_logger("");
    let mut ext = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let pairs = (0..accounts_len)
        .map(|i| address_build(i as u8))
        .collect::<Vec<_>>();

    let balances: Vec<_> = (0..accounts_len)
        .map(|i| (pairs[i].account_id.clone(), 1_000_000_000_000_000))
        .collect();

    pallet_balances::GenesisConfig::<Test> { balances }
        .assimilate_storage(&mut ext)
        .unwrap();

    let mut account_pairs = BTreeMap::new();
    for i in 0..pairs.len() {
        account_pairs.insert(pairs[i].address.clone(), pairs[i].account_id.clone());
    }

    let mut accounts = BTreeMap::new();
    for i in 0..pairs.len() {
        accounts.insert(
            pairs[i].address.clone(),
            pallet_evm::GenesisAccount {
                nonce: U256::default(),
                balance: U256::from_str("0xffffffffffffffffffffffffffffffff").unwrap(),
                storage: Default::default(),
                code: vec![
                    0x00, // STOP
                ],
            },
        );
    }

    pallet_evm::GenesisConfig::<Test> {
        account_pairs,
        accounts,
    }
    .assimilate_storage(&mut ext)
    .expect("Pallet balances storage can be assimilated");

    (pairs, ext.into())
}

pub fn contract_address(sender: H160, nonce: u64) -> H160 {
    let mut rlp = RlpStream::new_list(2);
    rlp.append(&sender);
    rlp.append(&nonce);

    H160::from_slice(&Keccak256::digest(&rlp.out())[12..])
}

pub fn storage_address(sender: H160, slot: H256) -> H256 {
    H256::from_slice(&Keccak256::digest(
        [&H256::from(sender)[..], &slot[..]].concat().as_slice(),
    ))
}

pub struct LegacyUnsignedTransaction {
    pub nonce: U256,
    pub gas_price: U256,
    pub gas_limit: U256,
    pub action: TransactionAction,
    pub value: U256,
    pub input: Vec<u8>,
}

impl LegacyUnsignedTransaction {
    fn signing_rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(9);
        s.append(&self.nonce);
        s.append(&self.gas_price);
        s.append(&self.gas_limit);
        s.append(&self.action);
        s.append(&self.value);
        s.append(&self.input);
        s.append(&ChainId::get());
        s.append(&0u8);
        s.append(&0u8);
    }

    fn signing_hash(&self) -> H256 {
        let mut stream = RlpStream::new();
        self.signing_rlp_append(&mut stream);
        H256::from_slice(&Keccak256::digest(&stream.out()).as_slice())
    }

    pub fn sign(&self, key: &H256) -> Transaction {
        self.sign_with_chain_id(key, ChainId::get())
    }

    pub fn sign_with_chain_id(&self, key: &H256, chain_id: u64) -> Transaction {
        let hash = self.signing_hash();
        let msg = libsecp256k1::Message::parse(hash.as_fixed_bytes());
        let s = libsecp256k1::sign(
            &msg,
            &libsecp256k1::SecretKey::parse_slice(&key[..]).unwrap(),
        );
        let sig = s.0.serialize();

        let sig = TransactionSignature::new(
            s.1.serialize() as u64 % 2 + chain_id * 2 + 35,
            H256::from_slice(&sig[0..32]),
            H256::from_slice(&sig[32..64]),
        )
        .unwrap();

        Transaction::Legacy(ethereum::LegacyTransaction {
            nonce: self.nonce,
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            action: self.action,
            value: self.value,
            input: self.input.clone(),
            signature: sig,
        })
    }
}

pub struct EIP2930UnsignedTransaction {
    pub nonce: U256,
    pub gas_price: U256,
    pub gas_limit: U256,
    pub action: TransactionAction,
    pub value: U256,
    pub input: Vec<u8>,
}

impl EIP2930UnsignedTransaction {
    pub fn sign(&self, secret: &H256, chain_id: Option<u64>) -> Transaction {
        let secret = {
            let mut sk: [u8; 32] = [0u8; 32];
            sk.copy_from_slice(&secret[0..]);
            libsecp256k1::SecretKey::parse(&sk).unwrap()
        };
        let chain_id = chain_id.unwrap_or(ChainId::get());
        let msg = ethereum::EIP2930TransactionMessage {
            chain_id: chain_id,
            nonce: self.nonce,
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            action: self.action,
            value: self.value,
            input: self.input.clone(),
            access_list: vec![],
        };
        let signing_message = libsecp256k1::Message::parse_slice(&msg.hash()[..]).unwrap();

        let (signature, recid) = libsecp256k1::sign(&signing_message, &secret);
        let rs = signature.serialize();
        let r = H256::from_slice(&rs[0..32]);
        let s = H256::from_slice(&rs[32..64]);
        Transaction::EIP2930(ethereum::EIP2930Transaction {
            chain_id: msg.chain_id,
            nonce: msg.nonce,
            gas_price: msg.gas_price,
            gas_limit: msg.gas_limit,
            action: msg.action,
            value: msg.value,
            input: msg.input.clone(),
            access_list: msg.access_list,
            odd_y_parity: recid.serialize() != 0,
            r,
            s,
        })
    }
}

pub struct EIP1559UnsignedTransaction {
    pub nonce: U256,
    pub max_priority_fee_per_gas: U256,
    pub max_fee_per_gas: U256,
    pub gas_limit: U256,
    pub action: TransactionAction,
    pub value: U256,
    pub input: Vec<u8>,
}

impl EIP1559UnsignedTransaction {
    pub fn sign(&self, secret: &H256, chain_id: Option<u64>) -> Transaction {
        let secret = {
            let mut sk: [u8; 32] = [0u8; 32];
            sk.copy_from_slice(&secret[0..]);
            libsecp256k1::SecretKey::parse(&sk).unwrap()
        };
        let chain_id = chain_id.unwrap_or(ChainId::get());
        let msg = ethereum::EIP1559TransactionMessage {
            chain_id: chain_id,
            nonce: self.nonce,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            gas_limit: self.gas_limit,
            action: self.action,
            value: self.value,
            input: self.input.clone(),
            access_list: vec![],
        };
        let signing_message = libsecp256k1::Message::parse_slice(&msg.hash()[..]).unwrap();

        let (signature, recid) = libsecp256k1::sign(&signing_message, &secret);
        let rs = signature.serialize();
        let r = H256::from_slice(&rs[0..32]);
        let s = H256::from_slice(&rs[32..64]);
        Transaction::EIP1559(ethereum::EIP1559Transaction {
            chain_id: msg.chain_id,
            nonce: msg.nonce,
            max_priority_fee_per_gas: msg.max_priority_fee_per_gas,
            max_fee_per_gas: msg.max_fee_per_gas,
            gas_limit: msg.gas_limit,
            action: msg.action,
            value: msg.value,
            input: msg.input.clone(),
            access_list: msg.access_list,
            odd_y_parity: recid.serialize() != 0,
            r,
            s,
        })
    }
}
