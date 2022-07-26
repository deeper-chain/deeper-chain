// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
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

//! Substrate chain configurations.

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use node_primitives::credit::{CreditData, CreditLevel, CreditSetting};
use node_runtime::constants::currency::*;
use node_runtime::Block;
use node_runtime::{
    wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, CreditConfig,
    DeeperNodeConfig, DemocracyConfig, EVMConfig, ElectionsConfig, EthereumConfig, GrandpaConfig,
    ImOnlineConfig, IndicesConfig, SessionConfig, SessionKeys, SocietyConfig, StakerStatus,
    StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_core::{H160, U256};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill, Percent,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

pub use node_primitives::{AccountId, Balance, BlockNumber, Signature};
pub use node_runtime::GenesisConfig;
use serde_json as json;

type AccountPublic = <Signature as Verify>::Signer;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
    /// The light sync state extension used by the sync-state rpc.
    pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    authority_endowment: Balance,
    stash: Balance,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    endowment: Balance,
    credit_settings: Vec<CreditSetting<Balance>>,
    user_credit_data: Vec<(AccountId, CreditData)>,
    _enable_println: bool,
) -> GenesisConfig {
    let mut accounts: BTreeSet<AccountId> = BTreeSet::new();
    let mut balances: Vec<(AccountId, Balance)> = vec![];
    for account in endowed_accounts.clone() {
        if !accounts.contains(&account) {
            balances.push((account.clone(), endowment));
            accounts.insert(account.clone());
        }
    }
    for authority in initial_authorities.clone() {
        if !accounts.contains(&authority.0) {
            balances.push((authority.0.clone(), stash));
            accounts.insert(authority.0.clone());
        }
        if !accounts.contains(&authority.1) {
            balances.push((authority.1.clone(), authority_endowment));
            accounts.insert(authority.1.clone());
        }
    }
    GenesisConfig {
        system: SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
        },
        balances: BalancesConfig { balances },
        indices: IndicesConfig { indices: vec![] },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        staking: StakingConfig {
            validator_count: initial_authorities.len() as u32,
            era_validator_reward: 57534 * DPR, // about 21 million DPR per year
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), stash, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        },
        democracy: DemocracyConfig::default(),
        elections: ElectionsConfig {
            members: initial_authorities
                .iter()
                .take((initial_authorities.len() + 1) / 2)
                .cloned()
                .map(|member| (member.0, stash))
                .collect(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((endowed_accounts.len() + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        },
        sudo: SudoConfig {
            key: Some(root_key),
        },
        babe: BabeConfig {
            authorities: vec![],
            epoch_config: Some(node_runtime::BABE_GENESIS_EPOCH_CONFIG),
        },
        im_online: ImOnlineConfig { keys: vec![] },
        authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        technical_membership: Default::default(),
        treasury: Default::default(),
        society: SocietyConfig {
            members: endowed_accounts
                .iter()
                .take((endowed_accounts.len() + 1) / 2)
                .cloned()
                .collect(),
            pot: 0,
            max_members: 999,
        },
        vesting: Default::default(),
        deeper_node: DeeperNodeConfig {
            reward_setting: vec![
                (
                    hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").into(),
                    H160::from_str("7a5b2024e179b312B924Ff02F4c27b5DF5326601")
                        .expect("internal H160 is valid; qed"),
                ),
                (
                    hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48").into(),
                    H160::from_str("120CF1Df8D02f6b1Aa4F2Dc9BF8FD7Cec63d8581")
                        .expect("internal H160 is valid; qed"),
                ),
                (
                    hex!("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20").into(),
                    H160::from_str("843AFB0DC3aD56696800C0d61C76Ac2A147AD48C")
                        .expect("internal H160 is valid; qed"),
                ),
                (
                    hex!("e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e").into(),
                    H160::from_str("e1bA6c4568D7ae1b87B9fF59eeB1d1Ff3c0C4f5B")
                        .expect("internal H160 is valid; qed"),
                ),
            ],
        },
        credit: CreditConfig {
            credit_settings,
            user_credit_data,
        },
        evm: EVMConfig {
            account_pairs: {
                let mut map = BTreeMap::new();
                // Alice's deeper chain address: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
                // H160 address of Alice mapping eth account
                // eth address: 0x7a5b2024e179b312B924Ff02F4c27b5DF5326601
                // eth privete key: 0xb52e6d24f6caacc1961d3cedf04ed3a11a7f4a27a6ce85eeea5dbea6c694f53a

                map.insert(
                    H160::from_str("7a5b2024e179b312B924Ff02F4c27b5DF5326601")
                        .expect("internal H160 is valid; qed"),
                    hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").into(),
                );
                // Bob's deeper chain address: 0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48
                // H160 address of Bob mapping eth account
                // eth address: 0x120CF1Df8D02f6b1Aa4F2Dc9BF8FD7Cec63d8581
                // eth privete key: 0xd2d7f189142e7a0468f97e5f16ef7762bf199a4c6c31b3e38fbf43f38f7d8f30

                map.insert(
                    H160::from_str("120CF1Df8D02f6b1Aa4F2Dc9BF8FD7Cec63d8581")
                        .expect("internal H160 is valid; qed"),
                    hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48").into(),
                );
                // Dave's deeper chain address: 0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20
                // H160 address of Dave mapping eth account
                // eth address: 0x843AFB0DC3aD56696800C0d61C76Ac2A147AD48C
                // eth privete key: 0xc32a1b133e8164ce6f63441090c29bd41e8ad0af1bf307c49ff3d40b1916db03
                map.insert(
                    H160::from_str("843AFB0DC3aD56696800C0d61C76Ac2A147AD48C")
                        .expect("internal H160 is valid; qed"),
                    hex!("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20").into(),
                );
                // Eve deeper chain address: 0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e
                // H160 address of Eve mapping eth account
                // eth address: 0xe1bA6c4568D7ae1b87B9fF59eeB1d1Ff3c0C4f5B
                // eth privete key: 0x828f8cdc56b6a78c5e9698900a00d7212780892da3486062d70092e3e9f6a37e
                map.insert(
                    H160::from_str("e1bA6c4568D7ae1b87B9fF59eeB1d1Ff3c0C4f5B")
                        .expect("internal H160 is valid; qed"),
                    hex!("e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e").into(),
                );
                map
            },
            accounts: {
                let mut map = BTreeMap::new();
                map.insert(
                    // H160 address of Alice eth account
                    H160::from_str("7a5b2024e179b312B924Ff02F4c27b5DF5326601")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map.insert(
                    // H160 address of Bob eth account
                    H160::from_str("120CF1Df8D02f6b1Aa4F2Dc9BF8FD7Cec63d8581")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );

                map.insert(
                    // H160 address of dave eth account
                    H160::from_str("843AFB0DC3aD56696800C0d61C76Ac2A147AD48C")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map.insert(
                    // H160 address of eve eth account
                    H160::from_str("e1bA6c4568D7ae1b87B9fF59eeB1d1Ff3c0C4f5B")
                        .expect("internal H160 is valid; qed"),
                    fp_evm::GenesisAccount {
                        balance: U256::from_str("0xffffffffffffffffffffffffffffffff")
                            .expect("internal U256 is valid; qed"),
                        code: Default::default(),
                        nonce: Default::default(),
                        storage: Default::default(),
                    },
                );
                map
            },
        },
        ethereum: EthereumConfig {},
        dynamic_fee: Default::default(),
        base_fee: Default::default(),
        council: Default::default(),
    }
}

fn campaign_0_credit_settings() -> Vec<CreditSetting<Balance>> {
    vec![
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Zero,
            staking_balance: 0,
            base_apy: Percent::from_percent(0),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 0,
            reward_per_referee: 0,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::One,
            staking_balance: 20_000 * DPR,
            base_apy: Percent::from_percent(39),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 1,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Two,
            staking_balance: 46_800 * DPR,
            base_apy: Percent::from_percent(47),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 2,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Three,
            staking_balance: 76_800 * DPR,
            base_apy: Percent::from_percent(53),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 3,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Four,
            staking_balance: 138_000 * DPR,
            base_apy: Percent::from_percent(59),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 7,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Five,
            staking_balance: 218_000 * DPR,
            base_apy: Percent::from_percent(66),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 12,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Six,
            staking_balance: 288_000 * DPR,
            base_apy: Percent::from_percent(74),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 18,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Seven,
            staking_balance: 368_000 * DPR,
            base_apy: Percent::from_percent(82),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 25,
            reward_per_referee: 0 * DPR,
        },
        CreditSetting {
            campaign_id: 0,
            credit_level: CreditLevel::Eight,
            staking_balance: 468_000 * DPR,
            base_apy: Percent::from_percent(90),
            bonus_apy: Percent::from_percent(0),
            max_rank_with_bonus: 0u32,
            tax_rate: Percent::from_percent(0),
            max_referees_with_rewards: 34,
            reward_per_referee: 0 * DPR,
        },
    ]
}

fn development_config_genesis() -> GenesisConfig {
    let endowed_accounts: Vec<AccountId> = vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
        get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
    ];

    let credit_settings = campaign_0_credit_settings();
    let user_credit_data = endowed_accounts
        .iter()
        .cloned()
        .map(|x| {
            (
                x,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 100,
                },
            )
        })
        .collect::<Vec<_>>();
    testnet_genesis(
        vec![authority_keys_from_seed("Alice")], // authorities
        10_000_000 * DPR,                        // authority endowment
        10_000 * DPR,                            // authority stash
        get_account_id_from_seed::<sr25519::Public>("Alice"), // root
        endowed_accounts,
        10_000_000 * DPR, // endowed accounts endowment
        credit_settings,
        user_credit_data,
        true,
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        development_config_genesis,
        vec![],
        None,
        None,
        None,
        Some(chain_spec_properties()),
        Default::default(),
    )
}

fn local_testnet_genesis() -> GenesisConfig {
    let authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        authority_keys_from_seed("Alice"),
        authority_keys_from_seed("Bob"),
    ];

    // 5CHu6tEdZWEnGHa928e9CfsXnL5otzRg4xGwqCscXDrcH38t
    let root_key: AccountId = get_account_id_from_seed::<sr25519::Public>("Alice");

    let endowed_accounts: Vec<AccountId> = vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
        get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
    ];
    let credit_settings = campaign_0_credit_settings();
    let user_credit_data = endowed_accounts
        .iter()
        .cloned()
        .map(|x| {
            (
                x,
                CreditData {
                    campaign_id: 0,
                    credit: 100,
                    initial_credit_level: CreditLevel::One,
                    rank_in_initial_credit_level: 1u32,
                    number_of_referees: 1,
                    current_credit_level: CreditLevel::One,
                    reward_eras: 100,
                },
            )
        })
        .collect::<Vec<_>>();
    testnet_genesis(
        authorities,
        10_000_000 * DPR, // authority endowment
        10_000 * DPR,     // authority stash
        root_key,
        endowed_accounts,
        10_000_000 * DPR, // endowed accounts endowment
        credit_settings,
        user_credit_data,
        true,
    )
}

/// customize tokenDecimals
pub fn chain_spec_properties() -> json::map::Map<String, json::Value> {
    let mut properties: json::map::Map<String, json::Value> = json::map::Map::new();
    properties.insert(
        String::from("ss58Format"),
        json::Value::Number(json::Number::from(42)),
    );
    properties.insert(
        String::from("tokenDecimals"),
        json::Value::Number(json::Number::from(18)),
    );
    properties.insert(
        String::from("tokenSymbol"),
        json::Value::String(String::from("DPR")),
    );
    properties
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        local_testnet_genesis,
        vec![],
        None,
        None,
        None,
        Some(chain_spec_properties()),
        Default::default(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::service::{new_full_base, NewFullBase};
    use sc_cli::SubstrateCli;
    use sc_service_test;
    use sp_runtime::BuildStorage;

    fn local_testnet_genesis_instant_single() -> GenesisConfig {
        let endowed_accounts: Vec<AccountId> = vec![];
        let credit_settings: Vec<CreditSetting<Balance>> = vec![];
        let user_credit_data: Vec<(AccountId, CreditData)> = vec![];
        testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            10_000_000 * DPR, // authority endowment
            10_000 * DPR,     // authority stash
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            endowed_accounts,
            10_000_000 * DPR, // endowed accounts endowment
            credit_settings,
            user_credit_data,
            false,
        )
    }

    /// Local testnet config (single validator - Alice)
    pub fn integration_test_config_with_single_authority() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            ChainType::Development,
            local_testnet_genesis_instant_single,
            vec![],
            None,
            None,
            None,
            None,
            Default::default(),
        )
    }

    /// Local testnet config (multivalidator Alice + Bob)
    pub fn integration_test_config_with_two_authorities() -> ChainSpec {
        ChainSpec::from_genesis(
            "Integration Test",
            "test",
            ChainType::Development,
            local_testnet_genesis,
            vec![],
            None,
            None,
            None,
            None,
            Default::default(),
        )
    }

    #[test]
    #[ignore]
    fn test_connectivity() {
        sp_tracing::try_init_simple();

        sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
            let cli = Cli::from_args();
            let NewFullBase {
                task_manager,
                client,
                network,
                transaction_pool,
                ..
            } = new_full_base(config, |_, _| (), &cli)?;
            Ok(sc_service_test::TestNetComponents::new(
                task_manager,
                client,
                network,
                transaction_pool,
            ))
        });
    }

    #[test]
    fn test_create_development_chain_spec() {
        development_config().build_storage().unwrap();
    }

    #[test]
    fn test_create_local_testnet_chain_spec() {
        local_testnet_config().build_storage().unwrap();
    }
}
