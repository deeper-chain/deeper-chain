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
use node_runtime::constants::currency::*;
use node_runtime::Block;
use node_runtime::{
    wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, BridgeConfig,
    ContractsConfig, CouncilConfig, CreditConfig, DeeperNodeConfig, DemocracyConfig,
    ElectionsConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig, SessionConfig, SessionKeys,
    SocietyConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
};
use pallet_credit::{CreditData, CreditLevel, CreditSetting};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill, Percent,
};

pub use node_primitives::{AccountId, Balance, BlockNumber, Signature};
pub use node_runtime::GenesisConfig;
use serde_json as json;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// get root key for deeper testnet
pub fn testnet_root_key() -> AccountId {
    hex![
        // 5CHu6tEdZWEnGHa928e9CfsXnL5otzRg4xGwqCscXDrcH38t
        "0a100b6bf4e332cac53b98af0003bbbf6984d2171bbe30a05a97bb28f5212119"
    ]
    .into()
}

/// return other authority keys as default validators
pub fn other_authority_keys() -> Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
)> {
    vec![
        (
            // 5CwMNoeXEktdpJFDNiPi29odWr8KANBWodzkBuE56DGa5ksq
            hex!["26a0928a4a88db828747ac4d503a902f279052aa6d48f1541bad709bbad1d750"].into(),
            // 5FyXGesEKhP7qKgx8GQs61hWF8HvrDCbCBTaN298SX3QTDhq
            hex!["acfd11cf17c7253febc403cf4c27d1ad673011f18c5aae8846eed067ae81d342"].into(),
            // 5EHzqtDmbUDZvxgGWjKGYz5kvmv1McBsBfZ3T2ZBL763yhj4
            hex!["629bd6b5e0bee300e455d2d5a367afca580cfbb7cab9856486c4fcc32ef9e825"]
                .unchecked_into(),
            // 5FyXGesEKhP7qKgx8GQs61hWF8HvrDCbCBTaN298SX3QTDhq
            hex!["acfd11cf17c7253febc403cf4c27d1ad673011f18c5aae8846eed067ae81d342"]
                .unchecked_into(),
            // 5FyXGesEKhP7qKgx8GQs61hWF8HvrDCbCBTaN298SX3QTDhq
            hex!["acfd11cf17c7253febc403cf4c27d1ad673011f18c5aae8846eed067ae81d342"]
                .unchecked_into(),
            // 5FyXGesEKhP7qKgx8GQs61hWF8HvrDCbCBTaN298SX3QTDhq
            hex!["acfd11cf17c7253febc403cf4c27d1ad673011f18c5aae8846eed067ae81d342"]
                .unchecked_into(),
        ),
        (
            // 5GQrjS6o6xG1LZxdc3SfVhoyCCCqBFL434seLiJLsJg92SyB
            hex!["c04fb7faed38acbb55f02afe12f624fc77a1b30e02ca8a6a08dde940baa9a82f"].into(),
            // 5GQpi5PnxBEBTzPwt8x4bYks1uD4Hy5A8ZxmmLihMiN3nqAA
            hex!["c048e845940a64de14307e316e987e95f4a199faf8ceb8d4e5a76f5f98f59c16"].into(),
            // 5DxYdPQuxWpjNWwPbzUE1QgkXJQR8NGjhQn7UuvD1Vaz4veX
            hex!["53c5ed4aec243acac4a02866f891f32653bc2ed54063eb5d9962ebdaa2dcdcbe"]
                .unchecked_into(),
            // 5GQpi5PnxBEBTzPwt8x4bYks1uD4Hy5A8ZxmmLihMiN3nqAA
            hex!["c048e845940a64de14307e316e987e95f4a199faf8ceb8d4e5a76f5f98f59c16"]
                .unchecked_into(),
            // 5GQpi5PnxBEBTzPwt8x4bYks1uD4Hy5A8ZxmmLihMiN3nqAA
            hex!["c048e845940a64de14307e316e987e95f4a199faf8ceb8d4e5a76f5f98f59c16"]
                .unchecked_into(),
            // 5GQpi5PnxBEBTzPwt8x4bYks1uD4Hy5A8ZxmmLihMiN3nqAA
            hex!["c048e845940a64de14307e316e987e95f4a199faf8ceb8d4e5a76f5f98f59c16"]
                .unchecked_into(),
        ),
        (
            // 5HNiABAGEcQtvtdkqrALzeieczDMAKjB4nEBqV7UcRsAEJxe
            hex!["eae899c4aac8bd52b2d206d244f26b6d39a7701939cbd33b2eafd11ca9050b0e"].into(),
            // 5Cd5bhgiBVAWxZsGiLUfC213A5cybeGgGShovY6ktKp5mosf
            hex!["18b10afafa9c3a3ac5ab3c886f68d7c13ac500fe009e9c35c9c2cc0188ad112f"].into(),
            // 5GFViwQFPAJqSw47jA9GmShvpG57kFEUku9GPJhR6EPNe6Ac
            hex!["b92bc9fcc24867030bb544e432e3a190a7516bde6008bcf3eeae6ec0c191fb8c"]
                .unchecked_into(),
            // 5Cd5bhgiBVAWxZsGiLUfC213A5cybeGgGShovY6ktKp5mosf
            hex!["18b10afafa9c3a3ac5ab3c886f68d7c13ac500fe009e9c35c9c2cc0188ad112f"]
                .unchecked_into(),
            // 5Cd5bhgiBVAWxZsGiLUfC213A5cybeGgGShovY6ktKp5mosf
            hex!["18b10afafa9c3a3ac5ab3c886f68d7c13ac500fe009e9c35c9c2cc0188ad112f"]
                .unchecked_into(),
            // 5Cd5bhgiBVAWxZsGiLUfC213A5cybeGgGShovY6ktKp5mosf
            hex!["18b10afafa9c3a3ac5ab3c886f68d7c13ac500fe009e9c35c9c2cc0188ad112f"]
                .unchecked_into(),
        ),
    ]
}

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
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

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

fn staging_testnet_config_genesis() -> GenesisConfig {
    // stash, controller, session-key
    // generated with secret:
    // for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
    // and
    // for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![
        (
            // 5Fbsd6WXDGiLTxunqeK5BATNiocfCqu9bS1yArVjCgeBLkVy
            hex!["9c7a2ee14e565db0c69f78c7b4cd839fbf52b607d867e9e9c5a79042898a0d12"].into(),
            // 5EnCiV7wSHeNhjW3FSUwiJNkcc2SBkPLn5Nj93FmbLtBjQUq
            hex!["781ead1e2fa9ccb74b44c19d29cb2a7a4b5be3972927ae98cd3877523976a276"].into(),
            // 5Fb9ayurnxnaXj56CjmyQLBiadfRCqUbL2VWNbbe1nZU6wiC
            hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"]
                .unchecked_into(),
            // 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
            hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
                .unchecked_into(),
            // 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
            hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
                .unchecked_into(),
            // 5EZaeQ8djPcq9pheJUhgerXQZt9YaHnMJpiHMRhwQeinqUW8
            hex!["6e7e4eb42cbd2e0ab4cae8708ce5509580b8c04d11f6758dbf686d50fe9f9106"]
                .unchecked_into(),
        ),
        (
            // 5ERawXCzCWkjVq3xz1W5KGNtVx2VdefvZ62Bw1FEuZW4Vny2
            hex!["68655684472b743e456907b398d3a44c113f189e56d1bbfd55e889e295dfde78"].into(),
            // 5Gc4vr42hH1uDZc93Nayk5G7i687bAQdHHc9unLuyeawHipF
            hex!["c8dc79e36b29395413399edaec3e20fcca7205fb19776ed8ddb25d6f427ec40e"].into(),
            // 5EockCXN6YkiNCDjpqqnbcqd4ad35nU4RmA1ikM4YeRN4WcE
            hex!["7932cff431e748892fa48e10c63c17d30f80ca42e4de3921e641249cd7fa3c2f"]
                .unchecked_into(),
            // 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
            hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
                .unchecked_into(),
            // 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
            hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
                .unchecked_into(),
            // 5DhLtiaQd1L1LU9jaNeeu9HJkP6eyg3BwXA7iNMzKm7qqruQ
            hex!["482dbd7297a39fa145c570552249c2ca9dd47e281f0c500c971b59c9dcdcd82e"]
                .unchecked_into(),
        ),
        (
            // 5DyVtKWPidondEu8iHZgi6Ffv9yrJJ1NDNLom3X9cTDi98qp
            hex!["547ff0ab649283a7ae01dbc2eb73932eba2fb09075e9485ff369082a2ff38d65"].into(),
            // 5FeD54vGVNpFX3PndHPXJ2MDakc462vBCD5mgtWRnWYCpZU9
            hex!["9e42241d7cd91d001773b0b616d523dd80e13c6c2cab860b1234ef1b9ffc1526"].into(),
            // 5E1jLYfLdUQKrFrtqoKgFrRvxM3oQPMbf6DfcsrugZZ5Bn8d
            hex!["5633b70b80a6c8bb16270f82cca6d56b27ed7b76c8fd5af2986a25a4788ce440"]
                .unchecked_into(),
            // 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
            hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
                .unchecked_into(),
            // 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
            hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
                .unchecked_into(),
            // 5DhKqkHRkndJu8vq7pi2Q5S3DfftWJHGxbEUNH43b46qNspH
            hex!["482a3389a6cf42d8ed83888cfd920fec738ea30f97e44699ada7323f08c3380a"]
                .unchecked_into(),
        ),
        (
            // 5HYZnKWe5FVZQ33ZRJK1rG3WaLMztxWrrNDb1JRwaHHVWyP9
            hex!["f26cdb14b5aec7b2789fd5ca80f979cef3761897ae1f37ffb3e154cbcc1c2663"].into(),
            // 5EPQdAQ39WQNLCRjWsCk5jErsCitHiY5ZmjfWzzbXDoAoYbn
            hex!["66bc1e5d275da50b72b15de072a2468a5ad414919ca9054d2695767cf650012f"].into(),
            // 5DMa31Hd5u1dwoRKgC4uvqyrdK45RHv3CpwvpUC1EzuwDit4
            hex!["3919132b851ef0fd2dae42a7e734fe547af5a6b809006100f48944d7fae8e8ef"]
                .unchecked_into(),
            // 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
            hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
                .unchecked_into(),
            // 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
            hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
                .unchecked_into(),
            // 5C4vDQxA8LTck2xJEy4Yg1hM9qjDt4LvTQaMo4Y8ne43aU6x
            hex!["00299981a2b92f878baaf5dbeba5c18d4e70f2a1fcd9c61b32ea18daf38f4378"]
                .unchecked_into(),
        ),
    ];

    // generated with secret: subkey inspect "$secret"/fir
    let root_key: AccountId = hex![
        // 5Ff3iXP75ruzroPWRP2FYBHWnmGGBSb63857BgnzCoXNxfPo
        "9ee5e5bdc0ec239eb164f865ecc345ce4c88e76ee002e0f7e318097347471809"
    ]
    .into();

    let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

    testnet_genesis(initial_authorities, root_key, Some(endowed_accounts), false)
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "Staging Testnet",
        "staging_testnet",
        ChainType::Live,
        staging_testnet_config_genesis,
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Staging telemetry url is valid; qed"),
        ),
        None,
        None,
        Default::default(),
    )
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
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    enable_println: bool,
) -> GenesisConfig {
    let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
        vec![
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
        ]
    });
    initial_authorities.iter().for_each(|x| {
        if !endowed_accounts.contains(&x.0) {
            endowed_accounts.push(x.0.clone())
        }
    });

    let num_endowed_accounts = endowed_accounts.len();

    const ENDOWMENT: Balance = 10_000_000 * DPR;
    const STASH: Balance = ENDOWMENT / 1000;

    let bridge_validators: Vec<AccountId> = vec![
        hex!("32b6e2fd3d19d875fc5a23a2bbc449b9b2dad1aa5f11aec6fe5ea9f5ba08f70e").into(),
        // 5DDCabfWypaJwMdXeKCxHmBtxWwob3RSYZeP9pMZa6V3bKEL
        hex!("9c164987ba60615be6074837036983ab96559cb4a3d6ada17ed0e092f044a521").into(),
        // 5FbMwvsF5serYgaQkcJ9itgiUX4RxftCF6reptrLym6YgERX
        hex!("5e414ecf3c9d3fba082d1b440b24abb7539ef64e9473bed53a754f686f06e52f").into(),
        // 5ECHkxssXVeENxozUbe4p64sZq6ktzFnv37BCbsAoS8AMxU3
    ];

    let mut new_endowed_accounts = endowed_accounts.clone();
    new_endowed_accounts
        .push(hex!("32b6e2fd3d19d875fc5a23a2bbc449b9b2dad1aa5f11aec6fe5ea9f5ba08f70e").into());
    new_endowed_accounts
        .push(hex!("9c164987ba60615be6074837036983ab96559cb4a3d6ada17ed0e092f044a521").into());
    new_endowed_accounts
        .push(hex!("5e414ecf3c9d3fba082d1b440b24abb7539ef64e9473bed53a754f686f06e52f").into());

    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: new_endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, ENDOWMENT))
                .collect(),
        }),
        pallet_indices: Some(IndicesConfig { indices: vec![] }),
        pallet_session: Some(SessionConfig {
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
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            era_validator_reward: 57534 * DPR, // about 21 million DPR per year
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, STASH))
                .collect(),
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        pallet_collective_Instance2: Some(TechnicalCommitteeConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            phantom: Default::default(),
        }),
        pallet_contracts: Some(ContractsConfig {
            current_schedule: pallet_contracts::Schedule {
                enable_println, // this should only be enabled on development chains
                ..Default::default()
            },
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_membership_Instance1: Some(Default::default()),
        pallet_treasury: Some(Default::default()),
        pallet_society: Some(SocietyConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .collect(),
            pot: 0,
            max_members: 999,
        }),
        pallet_vesting: Some(Default::default()),
        pallet_deeper_node: Some(DeeperNodeConfig { tmp: 0 }),
        pallet_eth_sub_bridge: Some(BridgeConfig {
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
        pallet_credit: Some(CreditConfig {
            credit_settings: vec![
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
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Two,
                    staking_balance: 46_800 * DPR,
                    base_apy: Percent::from_percent(40),
                    bonus_apy: Percent::from_percent(7),
                    max_rank_with_bonus: 1200u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 2,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Three,
                    staking_balance: 76_800 * DPR,
                    base_apy: Percent::from_percent(42),
                    bonus_apy: Percent::from_percent(11),
                    max_rank_with_bonus: 1000u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 3,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Four,
                    staking_balance: 138_000 * DPR,
                    base_apy: Percent::from_percent(46),
                    bonus_apy: Percent::from_percent(13),
                    max_rank_with_bonus: 800u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 7,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Five,
                    staking_balance: 218_000 * DPR,
                    base_apy: Percent::from_percent(50),
                    bonus_apy: Percent::from_percent(16),
                    max_rank_with_bonus: 600u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 12,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Six,
                    staking_balance: 288_000 * DPR,
                    base_apy: Percent::from_percent(54),
                    bonus_apy: Percent::from_percent(20),
                    max_rank_with_bonus: 400u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 18,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Seven,
                    staking_balance: 368_000 * DPR,
                    base_apy: Percent::from_percent(57),
                    bonus_apy: Percent::from_percent(25),
                    max_rank_with_bonus: 200u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 25,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 0,
                    credit_level: CreditLevel::Eight,
                    staking_balance: 468_000 * DPR,
                    base_apy: Percent::from_percent(60),
                    bonus_apy: Percent::from_percent(30),
                    max_rank_with_bonus: 100u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 34,
                    reward_per_referee: 18 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
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
                    campaign_id: 1,
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
                    campaign_id: 1,
                    credit_level: CreditLevel::Two,
                    staking_balance: 46_800 * DPR,
                    base_apy: Percent::from_percent(40),
                    bonus_apy: Percent::from_percent(4),
                    max_rank_with_bonus: 1200u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 2,
                    reward_per_referee: 0 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
                    credit_level: CreditLevel::Three,
                    staking_balance: 76_800 * DPR,
                    base_apy: Percent::from_percent(42),
                    bonus_apy: Percent::from_percent(8),
                    max_rank_with_bonus: 1000u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 3,
                    reward_per_referee: 0 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
                    credit_level: CreditLevel::Four,
                    staking_balance: 138_000 * DPR,
                    base_apy: Percent::from_percent(46),
                    bonus_apy: Percent::from_percent(10),
                    max_rank_with_bonus: 800u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 7,
                    reward_per_referee: 0 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
                    credit_level: CreditLevel::Five,
                    staking_balance: 218_000 * DPR,
                    base_apy: Percent::from_percent(50),
                    bonus_apy: Percent::from_percent(12),
                    max_rank_with_bonus: 600u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 12,
                    reward_per_referee: 0 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
                    credit_level: CreditLevel::Six,
                    staking_balance: 288_000 * DPR,
                    base_apy: Percent::from_percent(54),
                    bonus_apy: Percent::from_percent(15),
                    max_rank_with_bonus: 400u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 18,
                    reward_per_referee: 0 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
                    credit_level: CreditLevel::Seven,
                    staking_balance: 368_000 * DPR,
                    base_apy: Percent::from_percent(57),
                    bonus_apy: Percent::from_percent(18),
                    max_rank_with_bonus: 200u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 25,
                    reward_per_referee: 0 * DPR,
                },
                CreditSetting {
                    campaign_id: 1,
                    credit_level: CreditLevel::Eight,
                    staking_balance: 468_000 * DPR,
                    base_apy: Percent::from_percent(60),
                    bonus_apy: Percent::from_percent(20),
                    max_rank_with_bonus: 100u32,
                    tax_rate: Percent::from_percent(0),
                    max_referees_with_rewards: 34,
                    reward_per_referee: 0 * DPR,
                },
            ],
            user_credit_data: new_endowed_accounts
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
                            reward_eras: 1,
                            current_credit_level: CreditLevel::One,
                        },
                    )
                })
                .collect(),
        }),
    }
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![authority_keys_from_seed("Alice")],
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        None,
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
        Some(chain_spec_properties()),
        Default::default(),
    )
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            other_authority_keys()[0].clone(),
            other_authority_keys()[1].clone(),
            other_authority_keys()[2].clone(),
        ],
        testnet_root_key(),
        Some(vec![
            other_authority_keys()[0].1.clone(),
            other_authority_keys()[1].1.clone(),
            other_authority_keys()[2].1.clone(),
            testnet_root_key(),
            other_authority_keys()[0].0.clone(),
            other_authority_keys()[1].0.clone(),
            other_authority_keys()[2].0.clone(),
        ]),
        false,
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
        Some(chain_spec_properties()),
        Default::default(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::service::{new_full_base, new_light_base, NewFullBase};
    use sc_service_test;
    use sp_runtime::BuildStorage;

    fn local_testnet_genesis_instant_single() -> GenesisConfig {
        testnet_genesis(
            vec![authority_keys_from_seed("Alice")],
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            None,
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
            Default::default(),
        )
    }

    #[test]
    #[ignore]
    fn test_connectivity() {
        sc_service_test::connectivity(
            integration_test_config_with_two_authorities(),
            |config| {
                let NewFullBase {
                    task_manager,
                    client,
                    network,
                    transaction_pool,
                    ..
                } = new_full_base(config, |_, _| ())?;
                Ok(sc_service_test::TestNetComponents::new(
                    task_manager,
                    client,
                    network,
                    transaction_pool,
                ))
            },
            |config| {
                let (keep_alive, _, _, client, network, transaction_pool) = new_light_base(config)?;
                Ok(sc_service_test::TestNetComponents::new(
                    keep_alive,
                    client,
                    network,
                    transaction_pool,
                ))
            },
        );
    }

    #[test]
    fn test_create_development_chain_spec() {
        development_config().build_storage().unwrap();
    }

    #[test]
    fn test_create_local_testnet_chain_spec() {
        local_testnet_config().build_storage().unwrap();
    }

    #[test]
    fn test_staging_test_net_chain_spec() {
        staging_testnet_config().build_storage().unwrap();
    }
}
