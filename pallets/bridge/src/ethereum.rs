use std::str::FromStr;

use ethabi::{ethereum_types::H256, FunctionOutputDecoder};
use ethabi_contract::use_contract;
use hex::decode as hex_decode;
use hex_literal::hex;
use std::{future::Future, sync::Arc};
use serde::{Deserialize, Serialize};
use serde_json::Map;


#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransByHashResp {
    pub jsonrpc: &'static str,
    pub id: usize,
    pub result: Transaction,
    // pub method: &'static str,
    // pub params: Vec<&'static str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename="blockHash")]
    pub block_hash: String, // H160?
    
    #[serde(rename="blockNumber")]
    pub block_number: String, // H256?

    #[serde(rename="chainId")]
    pub chain_id: String,
    pub from: String,
    pub gas: String,

    #[serde(rename="gasPrice")]
    pub gas_price: String,
    pub hash: String,
    pub input: String,

    #[serde(rename="maxFeePerGas")]
    pub max_fee_per_gas: String,

    #[serde(rename="maxPriorityFeePerGas")]
    pub max_priority_fee_per_gas: String,
    pub nonce: String,
    pub r: String,
    pub s: String,
    pub to: String,

    #[serde(rename="transactionIndex")]
    pub transaction_index: String,
    #[serde(rename="type")]
    pub ty: String,
    pub v: String,
    pub value: String,
}


use_contract!(bridge_contract, "src/ether_bridge_abi.json");

fn decode_contract(input: String) -> bool {
    // bridge_contract::functions::total_supply::decode_output(input.as_bytes());
    let topics: Vec<H256> = vec![H256::from(hex!(
        "fb65d1544ea97e32c62baf55f738f7bb44671998c927415ef03e52d2477e292f"
    ))];
    let data = hex_decode(input).unwrap();
    let a = bridge_contract::events::relay_message::parse_log(ethabi::RawLog {
        topics: topics,
        data: data,
    });
    println!("{:?}", a.unwrap());

    let input_data = hex_decode("c031422600000000000000000000000000000000000000000000003635c9adc5dea00000c8d01af905315e01ec802d306a3ad4990ffe4acb94c0cf6c5b09a0bb32f3da45");
    let decoded_input = ethabi::decode(
        &[
            ethabi::ParamType::String,
            ethabi::ParamType::Uint(1),
            ethabi::ParamType::Address,
        ],
        &input_data.unwrap(),
    );
    println!("{:?}", decoded_input);
    true
}

#[test]
fn test_decode_contract() {
    let input1 = "e52dbf7803936e3fe1b06460bb89867f7c42fa35ad057f0a5255ce69a08b6292000000000000000000000000f16d1467458d667bdf8424861d510cf6383884abc8d01af905315e01ec802d306a3ad4990ffe4acb94c0cf6c5b09a0bb32f3da4500000000000000000000000000000000000000000000003635c9adc5dea00000".to_string();
    assert!(decode_contract(input1));
}
