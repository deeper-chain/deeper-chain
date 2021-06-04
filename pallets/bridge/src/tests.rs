use crate::mock::*;
use crate::types::Status;
use frame_support::{assert_noop, assert_ok};
use sp_core::{H160, H256};

const ETH_MESSAGE_ID: &[u8; 32] = b"0x5617efe391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID1: &[u8; 32] = b"0x5617iru391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID2: &[u8; 32] = b"0x5617yhk391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID3: &[u8; 32] = b"0x5617jdp391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID4: &[u8; 32] = b"0x5617kpt391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID5: &[u8; 32] = b"0x5617oet391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID6: &[u8; 32] = b"0x5617pey391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID7: &[u8; 32] = b"0x5617jqu391571b5dc8230db92ba65b";
const ETH_MESSAGE_ID8: &[u8; 32] = b"0x5617pbt391571b5dc8230db92ba65b";
const ETH_ADDRESS: &[u8; 20] = b"0x00b46c2526ebb8f4c9";
const V1: u64 = 1;
const V2: u64 = 2;
const V3: u64 = 3;
const V4: u64 = 4;
const USER1: u64 = 5;
const USER2: u64 = 6;
const USER3: u64 = 7;
const USER4: u64 = 8;
const USER5: u64 = 9;
const USER6: u64 = 10;
const USER7: u64 = 11;
const USER8: u64 = 12;
const USER9: u64 = 13;

#[test]
fn eth2sub_mint_works() {
    new_test_ext().execute_with(|| {
        let message_id = H256::from(ETH_MESSAGE_ID);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = 99;
        let balance_of_user2 = Balances::free_balance(USER2);
        let total_issuance = Balances::total_issuance();

        //substrate <----- ETH
        assert_ok!(BridgeModule::multi_signed_mint(
            Origin::signed(V2),
            message_id,
            eth_address,
            USER2,
            amount
        ));
        let mut message = BridgeModule::messages(message_id);
        assert_eq!(message.status, Status::Pending);

        assert_ok!(BridgeModule::multi_signed_mint(
            Origin::signed(V1),
            message_id,
            eth_address,
            USER2,
            amount
        ));
        message = BridgeModule::messages(message_id);
        assert_eq!(message.status, Status::Confirmed);

        let transfer = BridgeModule::transfers(0);
        assert_eq!(transfer.open, false);

        assert_eq!(Balances::free_balance(USER2), amount + balance_of_user2);
        assert_eq!(Balances::total_issuance(), amount + total_issuance);
    });
}

#[test]
fn eth2sub_closed_transfer_fail() {
    new_test_ext().execute_with(|| {
        let message_id = H256::from(ETH_MESSAGE_ID);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = 99;
        let balance_of_user2 = Balances::free_balance(USER2);
        let total_issuance = Balances::total_issuance();

        //substrate <----- ETH
        assert_ok!(BridgeModule::multi_signed_mint(
            Origin::signed(V2),
            message_id,
            eth_address,
            USER2,
            amount
        ));
        assert_ok!(BridgeModule::multi_signed_mint(
            Origin::signed(V1),
            message_id,
            eth_address,
            USER2,
            amount
        ));
        assert_noop!(
            BridgeModule::multi_signed_mint(
                Origin::signed(V3),
                message_id,
                eth_address,
                USER2,
                amount
            ),
            "This transfer is not open"
        );
        assert_eq!(Balances::free_balance(USER2), amount + balance_of_user2);
        assert_eq!(Balances::total_issuance(), amount + total_issuance);
        let transfer = BridgeModule::transfers(0);
        assert_eq!(transfer.open, false);

        let message = BridgeModule::messages(message_id);
        assert_eq!(message.status, Status::Confirmed);
    })
}

#[test]
fn sub2eth_burn_works() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;
        let balance_of_user2 = Balances::free_balance(USER2);
        let total_issuance = Balances::total_issuance();

        //substrate ----> ETH
        assert_ok!(BridgeModule::set_transfer(
            Origin::signed(USER2),
            eth_address,
            amount2
        ));
        //RelayMessage(message_id) event emitted

        let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
        let get_message = || BridgeModule::messages(sub_message_id);

        let mut message = get_message();
        assert_eq!(message.status, Status::Withdraw);

        //approval
        assert_eq!(Balances::free_balance(USER2), balance_of_user2);
        assert_ok!(BridgeModule::approve_transfer(
            Origin::signed(V1),
            sub_message_id
        ));
        assert_ok!(BridgeModule::approve_transfer(
            Origin::signed(V2),
            sub_message_id
        ));

        message = get_message();
        assert_eq!(message.status, Status::Approved);

        // at this point transfer is in Approved status and are waiting for confirmation
        // from ethereum side to burn. Funds are locked.

        assert_eq!(Balances::reserved_balance(USER2), amount2);

        assert_ok!(BridgeModule::confirm_transfer(
            Origin::signed(V2),
            sub_message_id
        ));

        message = get_message();
        let transfer = BridgeModule::transfers(1);
        assert_eq!(message.status, Status::Confirmed);
        assert_eq!(transfer.open, true);
        assert_ok!(BridgeModule::confirm_transfer(
            Origin::signed(V1),
            sub_message_id
        ));
        // assert_ok!(BridgeModule::confirm_transfer(Origin::signed(USER1), sub_message_id));
        //BurnedMessage(Hash, AccountId, H160, u64) event emitted
        assert_eq!(Balances::free_balance(USER2), balance_of_user2 - amount2);
        assert_eq!(Balances::total_issuance(), total_issuance - amount2);
    })
}

#[test]
fn sub2eth_burn_skipped_approval_should_fail() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 600;
        let amount2 = 49;

        //substrate ----> ETH
        assert_ok!(BridgeModule::set_transfer(
            Origin::signed(USER2),
            eth_address,
            amount2
        ));
        //RelayMessage(message_id) event emitted

        let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
        let message = BridgeModule::messages(sub_message_id);
        assert_eq!(message.status, Status::Withdraw);

        assert_eq!(Balances::reserved_balance(USER2), 0);
        // lets say validators blacked out and we
        // try to confirm without approval anyway
        assert_noop!(
            BridgeModule::confirm_transfer(Origin::signed(V1), sub_message_id),
            "This transfer must be approved first."
        );
    })
}

#[test]
fn sub2eth_burn_cancel_works() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 600;
        let amount2 = 49;

        //substrate ----> ETH
        assert_ok!(BridgeModule::set_transfer(
            Origin::signed(USER2),
            eth_address,
            amount2
        ));

        let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
        assert_ok!(BridgeModule::approve_transfer(
            Origin::signed(V1),
            sub_message_id
        ));
        assert_ok!(BridgeModule::approve_transfer(
            Origin::signed(V2),
            sub_message_id
        ));
        let mut message = BridgeModule::messages(sub_message_id);
        // funds are locked and waiting for confirmation
        assert_eq!(message.status, Status::Approved);
        assert_ok!(BridgeModule::cancel_transfer(
            Origin::signed(V2),
            sub_message_id
        ));
        assert_ok!(BridgeModule::cancel_transfer(
            Origin::signed(V3),
            sub_message_id
        ));
        message = BridgeModule::messages(sub_message_id);
        assert_eq!(message.status, Status::Canceled);
    })
}

#[test]
fn burn_cancel_should_fail() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;

        let balance_of_user2 = Balances::free_balance(USER2);
        let total_issuance = Balances::total_issuance();

        //substrate ----> ETH
        assert_ok!(BridgeModule::set_transfer(
            Origin::signed(USER2),
            eth_address,
            amount2
        ));

        let sub_message_id = BridgeModule::message_id_by_transfer_id(0);
        let get_message = || BridgeModule::messages(sub_message_id);

        let mut message = get_message();
        assert_eq!(message.status, Status::Withdraw);

        //approval
        assert_eq!(Balances::reserved_balance(USER2), 0);
        assert_ok!(BridgeModule::approve_transfer(
            Origin::signed(V1),
            sub_message_id
        ));
        assert_ok!(BridgeModule::approve_transfer(
            Origin::signed(V2),
            sub_message_id
        ));

        message = get_message();
        assert_eq!(message.status, Status::Approved);

        // at this point transfer is in Approved status and are waiting for confirmation
        // from ethereum side to burn. Funds are locked.
        assert_eq!(Balances::reserved_balance(USER2), amount2);
        assert_eq!(Balances::free_balance(USER2), balance_of_user2 - amount2);
        // once it happends, validators call confirm_transfer

        assert_ok!(BridgeModule::confirm_transfer(
            Origin::signed(V2),
            sub_message_id
        ));

        message = get_message();
        let transfer = BridgeModule::transfers(1);
        assert_eq!(message.status, Status::Confirmed);
        assert_eq!(transfer.open, true);
        assert_ok!(BridgeModule::confirm_transfer(
            Origin::signed(V1),
            sub_message_id
        ));
        // assert_ok!(BridgeModule::confirm_transfer(Origin::signed(USER1), sub_message_id));
        //BurnedMessage(Hash, AccountId, H160, u64) event emitted

        assert_eq!(Balances::free_balance(USER2), balance_of_user2 - amount2);
        assert_eq!(Balances::total_issuance(), total_issuance - amount2);
        assert_noop!(
            BridgeModule::cancel_transfer(Origin::signed(V2), sub_message_id),
            "Failed to cancel. This transfer is already executed."
        );
    })
}
