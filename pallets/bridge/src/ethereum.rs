use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Encode, Decode, Clone, PartialEq, Eq)]
pub struct SetTransferData {
    pub message_id: Vec<u8>, // bytes32
    // pub sender: Vec<u8>, // TODO: fix why can't just use FixedBytes(20)
    pub recipient: Vec<u8>, // bytes32
    pub amount: u128,
}

pub fn decode_data(data: &[u8]) -> SetTransferData {
    let decode_input = ethabi::decode(
        &[
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::FixedBytes(20),
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::Uint(128),
        ],
        data,
    )
    .unwrap();
    SetTransferData {
        message_id: decode_input[0].clone().into_fixed_bytes().unwrap(),
        // sender: decode_input[1].clone().into_fixed_bytes().unwrap(),
        recipient: decode_input[2].clone().into_fixed_bytes().unwrap(),
        amount: decode_input[3].clone().into_uint().unwrap().low_u128(),
    }
}

#[test]
fn test_decode_data() {
    use hex_literal::hex;
    // https://etherscan.io/tx/0x125906f92d35cf1b9586ced5557b8ce646e88353cdfe04336a546b8197a4c04a#eventlog
    assert_eq!(SetTransferData{
        message_id: hex!("3B63AD0B9C134CC87A287B22E17AB6475553EDEED90096C93E2C73ED58F49114").to_vec(),
        // sender: hex!("a56403cd96695f590638ba1af16a37f12d26f1f2").to_vec(),
        recipient: hex!("0E6409835F9B350D57FEAA750D1396A5493E2537BD72E908E2E24CE95DE47E3D").to_vec(),
        amount: 20000000000000000000000,
    }, decode_data(&hex!("3b63ad0b9c134cc87a287b22e17ab6475553edeed90096c93e2c73ed58f49114000000000000000000000000a56403cd96695f590638ba1af16a37f12d26f1f20e6409835f9b350d57feaa750d1396a5493e2537bd72e908e2e24ce95de47e3d00000000000000000000000000000000000000000000043c33c1937564800000")));
}

#[test]
fn test_deserilize() {
    let s = r#"{"jsonrpc":"2.0","id":1,"result":"0x10ff0e058a85e8b479fa3214d856f96b22fb9c28679a"}"#;
    let info: super::CreateNewFilterResp = serde_json::from_slice(s.as_bytes()).unwrap();
}
