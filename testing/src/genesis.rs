// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Genesis Configuration.

use crate::keyring::*;
use e2_chain_runtime::constants::currency::*;
use e2_chain_runtime::{
    wasm_binary_unwrap, AccountId, BalancesConfig, ContractsConfig, GenesisConfig, GrandpaConfig,
    IndicesConfig, SessionConfig, SocietyConfig, StakerStatus, StakingConfig, SystemConfig, BridgeConfig,
};
use sp_core::ChangesTrieConfiguration;
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use sp_runtime::Perbill;
use hex_literal::hex;

/// Create genesis runtime configuration for tests.
pub fn config(support_changes_trie: bool, code: Option<&[u8]>) -> GenesisConfig {
    config_endowed(support_changes_trie, code, Default::default())
}

/// Create genesis runtime configuration for tests with some extra
/// endowed accounts.
pub fn config_endowed(
    support_changes_trie: bool,
    code: Option<&[u8]>,
    extra_endowed: Vec<AccountId>,
) -> GenesisConfig {
    let mut endowed = vec![
        (alice(), 111 * DOLLARS),
        (bob(), 100 * DOLLARS),
        (charlie(), 100_000_000 * DOLLARS),
        (dave(), 111 * DOLLARS),
        (eve(), 101 * DOLLARS),
        (ferdie(), 100 * DOLLARS),
    ];

    let bridge_validators: Vec<AccountId> = vec![
        hex!("0d96d3dbdb55964e521a2f1dc1428ae55336063fd8f0e07bebbcb1becf79a67b").into(),
        // 5CtXvt2othnZpkneuTg6xENMwXbmwV3da1YeNAeYx5wMaCvz
        hex!("80133ea92f48aa928119aaaf524bc75e436a5c9eb24878a9e28ac7b0b37aa81a").into(), 
        // 5CqXmy44eTwGQCX8GaLrUfTAyEswGSd4PgSKMgUdLfDLBhZZ
        hex!("3c7f612cdda6d0a3aad9da0fb6cb624721b04067f00bd0034062e6e2db2cd23e").into(), 
        // 5DnUF5fQ6KNYPWRAcHYpMu32pUtdLv6ksRcSLeuofrxmPsTU
    ];
    endowed.extend(
        extra_endowed
            .into_iter()
            .map(|endowed| (endowed, 100 * DOLLARS)),
    );

    GenesisConfig {
        frame_system: Some(SystemConfig {
            changes_trie_config: if support_changes_trie {
                Some(ChangesTrieConfiguration {
                    digest_interval: 2,
                    digest_levels: 2,
                })
            } else {
                None
            },
            code: code
                .map(|x| x.to_vec())
                .unwrap_or_else(|| wasm_binary_unwrap().to_vec()),
        }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        pallet_balances: Some(BalancesConfig { balances: endowed }),
        pallet_session: Some(SessionConfig {
            keys: vec![
                (
                    dave(),
                    alice(),
                    to_session_keys(&Ed25519Keyring::Alice, &Sr25519Keyring::Alice),
                ),
                (
                    eve(),
                    bob(),
                    to_session_keys(&Ed25519Keyring::Bob, &Sr25519Keyring::Bob),
                ),
                (
                    ferdie(),
                    charlie(),
                    to_session_keys(&Ed25519Keyring::Charlie, &Sr25519Keyring::Charlie),
                ),
            ],
        }),
        deeper_node: Some(Default::default()),
        pallet_staking_with_credit: Some(StakingConfig {
            stakers: vec![
                (dave(), alice(), 111 * DOLLARS, StakerStatus::Validator),
                (eve(), bob(), 100 * DOLLARS, StakerStatus::Validator),
                (ferdie(), charlie(), 100 * DOLLARS, StakerStatus::Validator),
            ],
            validator_count: 3,
            minimum_validator_count: 0,
            slash_reward_fraction: Perbill::from_percent(10),
            invulnerables: vec![alice(), bob(), charlie()],
            ..Default::default()
        }),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: Default::default(),
        }),
        pallet_babe: Some(Default::default()),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(Default::default()),
        pallet_authority_discovery: Some(Default::default()),
        pallet_democracy: Some(Default::default()),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_collective_Instance2: Some(Default::default()),
        pallet_membership_Instance1: Some(Default::default()),
        pallet_elections_phragmen: Some(Default::default()),
        pallet_sudo: Some(Default::default()),
        pallet_treasury: Some(Default::default()),
        pallet_society: Some(SocietyConfig {
            members: vec![alice(), bob()],
            pot: 0,
            max_members: 999,
        }),
        pallet_vesting: Some(Default::default()),
        pallet_eth_sub_bridge: Some(BridgeConfig{
            validator_accounts: bridge_validators,
            validators_count: 3u32,
            current_limits: vec![
                100 * 10u128.pow(18),
                200 * 10u128.pow(18),
                50 * 10u128.pow(18),
                400 * 10u128.pow(18),
                10 * 10u128.pow(18),
            ],
        }),
    }
}
