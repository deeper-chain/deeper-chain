use core::convert::TryInto;
use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;
use sp_std::vec;

// pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur only when interacting with
/// an Ethereum node through RPC.
#[derive(Debug)]
pub enum Error {
    Network,
}

#[derive(Serialize, Deserialize, Debug, Default, Encode, Decode, Clone)]
pub struct GetLogsResp {
    pub result: Vec<EthLog>,
}

#[derive(Serialize, Deserialize, Debug, Default, Encode, Decode, Clone)]
pub struct EthLog {
    pub address: Vec<u8>,

    #[serde(rename = "blockHash")]
    pub block_hash: Vec<u8>,

    #[serde(rename = "blockNumber")]
    pub block_number: Vec<u8>,

    pub data: Vec<u8>,

    pub removed: bool,

    pub topics: Vec<Vec<u8>>,

    #[serde(rename = "transactionHash")]
    pub transaction_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct SetTransferData {
    pub message_id: Vec<u8>, // H256, bytes32
    pub sender: Vec<u8>,     // H160, Bytes20
    pub recipient: [u8; 32],  // H256, bytes32
    pub amount: u128,
}

pub trait Client {
    fn get_eth_logs(&self) -> Result<(Vec<SetTransferData>, Vec<u8>), Error>;
}

#[derive(Default)]
pub struct MockEthClient;

impl Client for MockEthClient {
    fn get_eth_logs(&self) -> Result<(Vec<SetTransferData>, Vec<u8>), Error> {
        Ok((vec![SetTransferData::default()], "abc".as_bytes().to_vec()))
    }
}
pub fn decode_data(data: &[u8]) -> SetTransferData {
    let decoded_data = ethabi::decode(
        &[
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::Address,
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::Uint(128),
        ],
        data,
    )
    .unwrap();
    SetTransferData {
        message_id: decoded_data[0].clone().into_fixed_bytes().unwrap(),
        sender: decoded_data[1]
            .clone()
            .into_address()
            .unwrap()
            .as_bytes()
            .to_vec(),
        recipient: decoded_data[2].clone().into_fixed_bytes().unwrap().try_into().unwrap(),
        amount: decoded_data[3].clone().into_uint().unwrap().low_u128(),
    }
}

#[test]
fn test_decode_data() {
    use hex_literal::hex;
    // https://etherscan.io/tx/0x125906f92d35cf1b9586ced5557b8ce646e88353cdfe04336a546b8197a4c04a#eventlog
    assert_eq!(SetTransferData{
        message_id: hex!("3B63AD0B9C134CC87A287B22E17AB6475553EDEED90096C93E2C73ED58F49114").to_vec(),
        sender: hex!("a56403cd96695f590638ba1af16a37f12d26f1f2").to_vec(),
        recipient: hex!("0E6409835F9B350D57FEAA750D1396A5493E2537BD72E908E2E24CE95DE47E3D").to_vec().try_into().unwrap(),
        amount: 20000000000000000000000,
    }, decode_data(&hex!("3b63ad0b9c134cc87a287b22e17ab6475553edeed90096c93e2c73ed58f49114000000000000000000000000a56403cd96695f590638ba1af16a37f12d26f1f20e6409835f9b350d57feaa750d1396a5493e2537bd72e908e2e24ce95de47e3d00000000000000000000000000000000000000000000043c33c1937564800000")));
}

#[test]
fn test_deserilize() {
    let s = r#"{"jsonrpc":"2.0","id":1,"result":"0x10ff0e058a85e8b479fa3214d856f96b22fb9c28679a"}"#;
    let filter_id = crate::pallet::parse_new_eth_filter_response(s);
    assert_eq!(
        "0x10ff0e058a85e8b479fa3214d856f96b22fb9c28679a"
            .as_bytes()
            .to_vec(),
        filter_id
    );
}
