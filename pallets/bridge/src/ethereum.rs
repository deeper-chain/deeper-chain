use codec::{Decode, Encode};
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use sp_runtime::offchain::{http, Duration};
use sp_std::prelude::*;
use sp_std::str;
use sp_std::vec;
use sp_std::vec::Vec;

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
    #[serde(with = "serde_bytes")]
    pub address: Vec<u8>,

    #[serde(rename = "blockNumber")]
    #[serde(with = "serde_bytes")]
    pub block_number: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct SetTransferData {
    pub message_id: Vec<u8>, // H256, bytes32
    pub sender: Vec<u8>,     // H160, Bytes20
    pub recipient: [u8; 32], // H256, bytes32
    pub amount: u128,
}

pub trait Client {
    fn get_eth_logs(from_block_hash: Vec<u8>) -> Result<(Vec<SetTransferData>, Vec<u8>), Error>;
}

// A mock eth client for test, in the future should use a real client
#[derive(Default)]
pub struct MockEthClient;

impl Client for MockEthClient {
    fn get_eth_logs(from_block_hash: Vec<u8>) -> Result<(Vec<SetTransferData>, Vec<u8>), Error> {
        Ok((vec![SetTransferData::default()], from_block_hash))
    }
}

#[derive(Default)]
pub struct RealEthClient;

impl Client for RealEthClient {
    fn get_eth_logs(
        mut from_block_number: Vec<u8>,
    ) -> Result<(Vec<SetTransferData>, Vec<u8>), Error> {
        if from_block_number.is_empty() {
            from_block_number = "earliest".as_bytes().to_vec();
        }
        let (logs, from_block_number) = Self::get_eth_logs_n_parse(from_block_number.clone())?;
        Ok((logs, from_block_number))
    }
}

pub const HTTP_REMOTE_REQUEST: &str = "https://kovan.infura.io/v3/bd22e70259d546dd832b63b7cab12ed0";
const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds

impl RealEthClient {
    fn get_eth_logs_n_parse(
        mut from_block_number: Vec<u8>,
    ) -> Result<(Vec<SetTransferData>, Vec<u8>), Error> {
        let resp_bytes =
            Self::get_eth_logs_from_remote(from_block_number.clone()).map_err(|e| {
                log::error!("fetch_from_remote error: {:?}", e);
                Error::Network
            })?;

        let resp_str = str::from_utf8(&resp_bytes).map_err(|_| Error::Network)?;
        log::info!("get_eth_logs_from_remote: {}", resp_str);

        // Deserializing JSON to struct, thanks to `serde` and `serde_derive`
        let info: GetLogsResp = serde_json::from_str(&resp_str).map_err(|e| {
            log::error!("deserialize_err, {:?}", e);
            Error::Network
        })?;
        let data: Vec<SetTransferData> = info.result.iter().map(|d| decode_data(&d.data)).collect();
        if let Some(last_block) = info.result.iter().last() {
            from_block_number = last_block.block_number.clone();
        }
        log::info!("get_eth_logs_n_parse: {:?}, {:?}", data, from_block_number);

        Ok((data, from_block_number))
    }

    fn get_eth_logs_from_remote(from_block_number: Vec<u8>) -> Result<Vec<u8>, Error> {
        let address = "0x309ed2c169decb81c5969dc9667aa7919170b6bd";
        let body_str = r#"
        {
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [
                {
                    "address": "ETH_GETLOGS_ADDRESS",
                    "topics": [
                        "0xfb65d1544ea97e32c62baf55f738f7bb44671998c927415ef03e52d2477e292f"
                    ],
                    "fromBlock": "ETH_GETLOGS_FROM_BLOCK_NUMBER",
                    "toBlock": "latest"
                }
            ],
            "id": 1
        }"#;
        let body_str = body_str.replace(
            "ETH_GETLOGS_FROM_BLOCK_NUMBER",
            str::from_utf8(&from_block_number).unwrap(),
        );
        let body_str = body_str.replace("ETH_GETLOGS_ADDRESS", address);

        let body = vec![body_str.as_str()];
        log::info!("get_eth_logs_from_remote: {:?}", body);

        let request = http::Request::post(HTTP_REMOTE_REQUEST, body);

        // Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
        let timeout = sp_io::offchain::timestamp().add(Duration::from_millis(FETCH_TIMEOUT_PERIOD));

        let pending = request
            .add_header("Content-Type", "application/json")
            .deadline(timeout) // Setting the timeout time
            .send() // Sending the request out by the host
            .map_err(|e| {
                log::error!("{:?}", e);
                Error::Network
            })?;

        let response = pending
            .try_wait(timeout)
            .map_err(|e| {
                log::error!("{:?}", e);
                Error::Network
            })?
            .map_err(|e| {
                log::error!("{:?}", e);
                Error::Network
            })?;

        if response.code != 200 {
            log::error!("Unexpected http request status code: {}", response.code);
            return Err(Error::Network);
        }

        // Next we fully read the response body and collect it to a vector of bytes.
        Ok(response.body().collect::<Vec<u8>>())
    }
}

// parse response of new_filter into a struct is hard in no_std, so use a
// string matching to get the filter_id
pub fn parse_new_eth_filter_response(resp_str: &str) -> Vec<u8> {
    if let Some(pos) = resp_str.find("result") {
        let start = pos + 9;
        let end = start + 46;
        let result = &resp_str[start..end];
        return result.as_bytes().to_vec();
    }
    vec![]
}

pub fn decode_data(data: &[u8]) -> SetTransferData {
    let mut origin_str = str::from_utf8(data).unwrap();
    if origin_str.starts_with("0x") {
        origin_str = &origin_str[2..];
    }
    let data_arr = hex::decode(origin_str).unwrap();
    let decoded_data = ethabi::decode(
        &[
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::Address,
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::Uint(128),
        ],
        &data_arr,
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
        recipient: decoded_data[2]
            .clone()
            .into_fixed_bytes()
            .unwrap()
            .try_into()
            .unwrap(),
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
    }, decode_data("0x3b63ad0b9c134cc87a287b22e17ab6475553edeed90096c93e2c73ed58f49114000000000000000000000000a56403cd96695f590638ba1af16a37f12d26f1f20e6409835f9b350d57feaa750d1396a5493e2537bd72e908e2e24ce95de47e3d00000000000000000000000000000000000000000000043c33c1937564800000".as_bytes()));

    // https://kovan.etherscan.io/tx/0xbf7709e510376ccc969562b8fa3d2bfbfed5093c35143e81f9b2780147731c41#eventlog
    assert_eq!(SetTransferData {
        message_id: hex!("80C4241D9D0C28C6D647DE0DD7D4099E32C5B4114EA9C4C74EFBC3C2C317C3EC").to_vec(),
        sender: hex!("720ac46fdb6da28fa751bc60afb8094290c2b4b7").to_vec(),
        recipient: hex!("C2F9A45E09CF0943E8A9CED81426842CCB640A55BD54E3A3D5F6186C5F584216").to_vec().try_into().unwrap(),
        amount: 1000000000000000000000,
    }, decode_data("0x80c4241d9d0c28c6d647de0dd7d4099e32c5b4114ea9c4c74efbc3c2c317c3ec000000000000000000000000720ac46fdb6da28fa751bc60afb8094290c2b4b7c2f9a45e09cf0943e8a9ced81426842ccb640a55bd54e3a3d5f6186c5f58421600000000000000000000000000000000000000000000003635c9adc5dea00000".as_bytes()))
}

#[test]
fn test_deserilize() {
    let s = r#"{"jsonrpc":"2.0","id":1,"result":"0x10ff0e058a85e8b479fa3214d856f96b22fb9c28679a"}"#;
    let filter_id = crate::ethereum::parse_new_eth_filter_response(s);
    assert_eq!(
        "0x10ff0e058a85e8b479fa3214d856f96b22fb9c28679a"
            .as_bytes()
            .to_vec(),
        filter_id
    );
}
