use crate::{mock::*, Error};
use sp_core::sr25519::{Public, Signature};
use sp_io::crypto::sr25519_verify;
use frame_support::assert_ok;
use sp_runtime::DispatchError;
//use sp_keyring::sr25519::Keyring;
//use frame_system::ensure_signed;


#[test]
fn fn_open_channel() {
    new_test_ext().execute_with(|| {
        // OK
        assert_ok!(Micropayment::open_channel(Origin::signed(1), 2, 300, 3600));

        // Channel already opened
        assert_eq!(
            Micropayment::open_channel(Origin::signed(1), 2, 1000, 3600), 
            Err(DispatchError::from(Error::<Test>::ChannelAlreadyOpened))
        );
        
        // Channel should connect two different accounts
        assert_eq!(
            Micropayment::open_channel(Origin::signed(2), 2, 1000, 3600), 
            Err(DispatchError::from(Error::<Test>::SameChannelEnds))
        );

        //  duration should > 0
        assert_eq!(
            Micropayment::open_channel(Origin::signed(3), 4, 200, 0), 
            Ok(())
        );

        // balance of 2 is 500, but channel balance experted is 1000  
        assert_eq!(
            Micropayment::open_channel(Origin::signed(2), 3, 1000, 3600), 
            Err(DispatchError::Module { index: 0, error: 0, message: Some("NotEnoughBalance") })
        );

    });
}


#[test]
fn fn_close_channel() {
    new_test_ext().execute_with(|| {
        // OK
        assert_ok!(Micropayment::open_channel(Origin::signed(1), 2, 300, 3600));
        assert_ok!(Micropayment::close_channel(Origin::signed(2), 1));

        // Channel not exists
        assert_eq!(
            Micropayment::close_channel(Origin::signed(2), 3), 
            Err(DispatchError::from(Error::<Test>::ChannelNotExist))
        );
        assert_eq!(
            Micropayment::close_channel(Origin::signed(1), 2), 
            Err(DispatchError::from(Error::<Test>::ChannelNotExist))
        );
    });
}

#[test]
fn fn_add_balance() {
    new_test_ext().execute_with(|| {
        // OK
        assert_ok!(Micropayment::open_channel(Origin::signed(1), 2, 300, 3600));
        assert_ok!(Micropayment::add_balance(Origin::signed(1), 2, 100));

        // Channel not exists
        assert_eq!(
            Micropayment::add_balance(Origin::signed(2), 3, 100), 
            Err(DispatchError::Module { index: 0, error: 1, message: Some("ChannelNotExist") })
        );

        // NotEnoughBalance 500-300 = 200, but add_balance 500
        assert_ok!(Micropayment::open_channel(Origin::signed(3), 4, 300, 3600));
        assert_eq!(
            Micropayment::add_balance(Origin::signed(3), 4, 500), 
            Err(DispatchError::Module { index: 0, error: 0, message: Some("NotEnoughBalance") })
        );

    });
}

#[test]
fn test_blake2_hash() {
    let bob: [u8; 32] = [
        142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147,
        201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
    ];
    let session_id: u32 = 22;
    let amount: u128 = 100;
    let mut data = Vec::new();

    let should_be: [u8; 32] = [
        204, 32, 30, 136, 139, 38, 43, 64, 99, 194, 191, 149, 97, 108, 87, 173, 224, 25, 104, 100,
        0, 179, 72, 91, 202, 84, 34, 190, 178, 119, 59, 41,
    ];

    data.extend_from_slice(&bob);
    data.extend_from_slice(&session_id.to_be_bytes());
    data.extend_from_slice(&amount.to_be_bytes());
    let hash = sp_io::hashing::blake2_256(&data);
    assert_eq!(&hash, &should_be);
}

#[test]
fn test_signature() {
    let sig: [u8; 64] = [
        68, 47, 70, 69, 17, 14, 9, 253, 233, 25, 253, 31, 54, 87, 196, 88, 192, 81, 241, 235, 51,
        175, 232, 189, 181, 176, 89, 123, 223, 237, 162, 39, 79, 234, 237, 116, 157, 88, 19, 64,
        224, 90, 66, 80, 4, 202, 207, 153, 220, 159, 142, 118, 210, 8, 25, 102, 159, 44, 229, 1,
        58, 237, 243, 135,
    ];
    assert_eq!(sig.len(), 64);
    let pk: [u8; 32] = [
        212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88,
        133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ];
    assert_eq!(pk.len(), 32);
    let msg: [u8; 32] = [
        204, 32, 30, 136, 139, 38, 43, 64, 99, 194, 191, 149, 97, 108, 87, 173, 224, 25, 104, 100,
        0, 179, 72, 91, 202, 84, 34, 190, 178, 119, 59, 41,
    ];

    let pk = Public::from_raw(pk);
    let sig = Signature::from_slice(&sig);
    println!("pk:{:?}", pk);
    println!("sig:{:?}", sig);
    println!("msg:{:?}", msg);
    let verified = sr25519_verify(&sig, &msg, &pk);
    assert_eq!(verified, true);
}

#[test]
fn fn_update_micropayment_information() {
    new_test_ext().execute_with(|| {
        Micropayment::update_micropayment_information(&2, &1, 5);
        let balance =
            Micropayment::get_clientpayment_by_block_server((&System::block_number(), 1), 2);
        Micropayment::update_micropayment_information(&2, &1, 5);
        assert_eq!(balance, 5);
        let balance1 =
            Micropayment::get_clientpayment_by_block_server((&System::block_number(), 1), 2);
        assert_eq!(balance1, 10);
        let server_flag = Micropayment::get_server_by_block(&System::block_number(), &1);
        let client_flag = Micropayment::get_server_by_block(&System::block_number(), &2);
        assert_eq!(server_flag, true);
        assert_eq!(client_flag, false);
    });
}

#[test]
fn fn_micropayment_statistics() {
    new_test_ext().execute_with(|| {
        Micropayment::update_micropayment_information(&2, &1, 7);
        Micropayment::update_micropayment_information(&4, &1, 3);
        run_to_block(3);
        Micropayment::update_micropayment_information(&1, &2, 1);
        Micropayment::update_micropayment_information(&3, &1, 8);
        run_to_block(6);
        Micropayment::update_micropayment_information(&3, &2, 9);
        run_to_block(9);
        let mut res = Micropayment::micropayment_statistics(0, 9);
        res.sort_by(|a, b| (*a).0.cmp(&(*b).0));
        assert_eq!(res[0], (1, 18, 3));
        assert_eq!(res[1], (2, 10, 2));
    });
}
