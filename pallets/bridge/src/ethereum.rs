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
    pub recipient: [u8; 32], // H256, bytes32
    pub amount: u128,
}

pub trait Client {
    fn get_eth_logs(filter_id: Vec<u8>) -> Result<(Vec<SetTransferData>, Vec<u8>), Error>;
}

// A mock eth client for test, in the future should use a real client
#[derive(Default)]
pub struct MockEthClient;

impl Client for MockEthClient {
    fn get_eth_logs(filter_id: Vec<u8>) -> Result<(Vec<SetTransferData>, Vec<u8>), Error> {
        Ok((vec![SetTransferData::default()], filter_id))
    }
}

#[derive(Default)]
pub struct RealEthClient;

impl Client for RealEthClient {
    fn get_eth_logs(mut filter_id: Vec<u8>) -> Result<(Vec<SetTransferData>, Vec<u8>), Error> {
        if filter_id.is_empty() {
            filter_id = Self::create_eth_filter_id_n_parse()?;
        }
        log::info!("before get_eth_logs_n_parse: {:?}", filter_id);
        let logs = Self::get_eth_logs_n_parse(filter_id.clone())?;
        Ok((logs, filter_id))
    }
}

const HTTP_REMOTE_REQUEST: &str = "https://mainnet.infura.io/v3/75284d8d0fb14ab88520b949270fe205";
const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds

impl RealEthClient {
    fn get_eth_logs_n_parse(filter_id: Vec<u8>) -> Result<Vec<SetTransferData>, Error> {
        let resp_bytes = Self::get_eth_logs_from_remote(filter_id).map_err(|e| {
            log::error!("fetch_from_remote error: {:?}", e);
            Error::Network
        })?;

        let resp_str = str::from_utf8(&resp_bytes).map_err(|_| Error::Network)?;
        log::info!("get_eth_logs_n_parse: {}", resp_str);

        // Deserializing JSON to struct, thanks to `serde` and `serde_derive`
        let info: GetLogsResp = serde_json::from_str(&resp_str).map_err(|_| Error::Network)?;
        let data: Vec<SetTransferData> = info.result.iter().map(|d| decode_data(&d.data)).collect();
        Ok(data)
    }

    fn get_eth_logs_from_remote(mut filter_id: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut body_bytes =
            "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getFilterChanges\",\"params\":[\""
                .as_bytes()
                .to_vec();
        body_bytes.append(&mut filter_id);
        body_bytes.append(&mut "\"],\"id\":1}".as_bytes().to_vec());
        let body_str = str::from_utf8(&body_bytes).unwrap();
        let body = vec![body_str];
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

    fn create_eth_filter_id_from_remote() -> Result<Vec<u8>, Error> {
        let body = vec![
            r#"{"jsonrpc":"2.0","method":"eth_newFilter","params":[{"topics": ["0xfb65d1544ea97e32c62baf55f738f7bb44671998c927415ef03e52d2477e292f"]}],"id":1}"#,
        ];
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

        // By default, the http request is async from the runtime perspective. So we are asking the
        //   runtime to wait here
        // The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
        //   ref: https://docs.substrate.io/rustdocs/latest/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
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
    fn create_eth_filter_id_n_parse() -> Result<Vec<u8>, Error> {
        let resp_bytes = Self::create_eth_filter_id_from_remote().map_err(|e| {
            log::error!("create_new_filter_id_n_parse error: {:?}", e);
            Error::Network
        })?;

        let resp_str = str::from_utf8(&resp_bytes).map_err(|_| Error::Network)?;
        log::info!("create_new_filter_id_n_parse: {}", resp_str);

        let filter_id = parse_new_eth_filter_response(resp_str);
        if !filter_id.is_empty() {
            return Ok(filter_id);
        }
        Err(Error::Network)
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
    }, decode_data(&hex!("3b63ad0b9c134cc87a287b22e17ab6475553edeed90096c93e2c73ed58f49114000000000000000000000000a56403cd96695f590638ba1af16a37f12d26f1f20e6409835f9b350d57feaa750d1396a5493e2537bd72e908e2e24ce95de47e3d00000000000000000000000000000000000000000000043c33c1937564800000")));
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
