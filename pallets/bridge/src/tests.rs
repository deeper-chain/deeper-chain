use crate::{mock::*, types::Status};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo};
use sp_core::{H160, H256, sr25519};
// use crate::mock::USER2();

const DAY_IN_BLOCKS: u32 = 14_400;

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

// const V1(): u64 = 1;
// const V2(): u64 = 2;
// const V3(): u64 = 3;
// const V4(): u64 = 4;

// const USER1(): u64 = 5;
// const USER2(): u64 = 6;
// const USER3(): u64 = 7;
// const USER4(): u64 = 8;
// const USER5(): u64 = 9;
// const USER6(): u64 = 10;
// const USER7(): u64 = 11;
// const USER8(): u64 = 12;
// const USER9(): u64 = 13;

#[test]
fn eth2sub_mint_works() {
    new_test_ext().execute_with(|| {
        let message_id = H256::from(ETH_MESSAGE_ID);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount = 99;
        let balance_of_user2 = Balances::free_balance(USER2());
        let total_issuance = Balances::total_issuance();

        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            message_id,
            eth_address,
            USER2(),
            amount
        ));
        let mut message = Bridge::messages(message_id);
        assert_eq!(message.status, Status::Pending);

        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V1()),
            message_id,
            eth_address,
            USER2(),
            amount
        ));
        message = Bridge::messages(message_id);
        assert_eq!(message.status, Status::Confirmed);

        let transfer = Bridge::transfers(0);
        assert_eq!(transfer.open, false);

        assert_eq!(Balances::free_balance(USER2()), amount + balance_of_user2);
        assert_eq!(Balances::total_issuance(), amount + total_issuance);
    });
}

#[test]
fn eth2sub_mint_should_fail() {
    new_test_ext().execute_with(|| {
        let message_id = H256::from(ETH_MESSAGE_ID);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 9;
        let amount2 = 101;

        assert_eq!(
            Bridge::multi_signed_mint(Origin::signed(V2()), message_id, eth_address, USER2(), amount1),
            Err(DispatchErrorWithPostInfo::from(
                "Invalid amount for transaction. Reached minimum limit."
            ))
        );

        assert_eq!(
            Bridge::multi_signed_mint(Origin::signed(V1()), message_id, eth_address, USER2(), amount2),
            Err(DispatchErrorWithPostInfo::from(
                "Invalid amount for transaction. Reached maximum limit."
            ))
        );
    });
}

#[test]
fn sub2eth_burn_works() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;
        let balance_of_user2 = Balances::free_balance(USER2());
        let total_issuance = Balances::total_issuance();

        //substrate ----> ETH
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));
        //RelayMessage(message_id) event emitted

        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        let get_message = || Bridge::messages(sub_message_id);

        let mut message = get_message();
        assert_eq!(message.status, Status::Withdraw);

        //approval
        assert_eq!(Balances::free_balance(USER2()), balance_of_user2);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));
        assert_ok!(Bridge::approve_transfer(Origin::signed(V2()), sub_message_id));

        message = get_message();
        assert_eq!(message.status, Status::Approved);

        // at this point transfer is in Approved status and are waiting for confirmation
        // from ethereum side to burn. Funds are locked.

        assert_eq!(Balances::reserved_balance(USER2()), amount2);

        assert_ok!(Bridge::confirm_transfer(Origin::signed(V2()), sub_message_id));

        message = get_message();
        let transfer = Bridge::transfers(1);
        assert_eq!(message.status, Status::Confirmed);
        assert_eq!(transfer.open, true);
        assert_ok!(Bridge::confirm_transfer(Origin::signed(V1()), sub_message_id));
        // assert_ok!(Bridge::confirm_transfer(Origin::signed(USER1()), sub_message_id));
        //BurnedMessage(Hash, AccountId, H160, u64) event emitted
        assert_eq!(Balances::free_balance(USER2()), balance_of_user2 - amount2);
        assert_eq!(Balances::total_issuance(), total_issuance - amount2);
    })
}

#[test]
fn sub2eth_burn_skipped_approval_should_fail() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;

        //substrate ----> ETH
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));
        //RelayMessage(message_id) event emitted

        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        let message = Bridge::messages(sub_message_id);
        assert_eq!(message.status, Status::Withdraw);

        assert_eq!(Balances::reserved_balance(USER2()), 0);
        // lets say validators blacked out and we
        // try to confirm without approval anyway
        assert_noop!(
            Bridge::confirm_transfer(Origin::signed(V1()), sub_message_id),
            "This transfer must be approved first."
        );
    })
}

#[test]
fn sub2eth_burn_cancel_works() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;

        //substrate ----> ETH
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));

        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));
        assert_ok!(Bridge::approve_transfer(Origin::signed(V2()), sub_message_id));
        let mut message = Bridge::messages(sub_message_id);
        // funds are locked and waiting for confirmation
        assert_eq!(message.status, Status::Approved);
        assert_ok!(Bridge::cancel_transfer(Origin::signed(V2()), sub_message_id));
        assert_ok!(Bridge::cancel_transfer(Origin::signed(V3()), sub_message_id));
        message = Bridge::messages(sub_message_id);
        assert_eq!(message.status, Status::Canceled);
    })
}

#[test]
fn burn_cancel_should_fail() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;

        let balance_of_user2 = Balances::free_balance(USER2());
        let total_issuance = Balances::total_issuance();

        //substrate ----> ETH
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));

        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        let get_message = || Bridge::messages(sub_message_id);

        let mut message = get_message();
        assert_eq!(message.status, Status::Withdraw);

        //approval
        assert_eq!(Balances::reserved_balance(USER2()), 0);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));
        assert_ok!(Bridge::approve_transfer(Origin::signed(V2()), sub_message_id));

        message = get_message();
        assert_eq!(message.status, Status::Approved);

        // at this point transfer is in Approved status and are waiting for confirmation
        // from ethereum side to burn. Funds are locked.
        assert_eq!(Balances::reserved_balance(USER2()), amount2);
        assert_eq!(Balances::free_balance(USER2()), balance_of_user2 - amount2);
        // once it happends, validators call confirm_transfer

        assert_ok!(Bridge::confirm_transfer(Origin::signed(V2()), sub_message_id));

        message = get_message();
        let transfer = Bridge::transfers(1);
        assert_eq!(message.status, Status::Confirmed);
        assert_eq!(transfer.open, true);
        assert_ok!(Bridge::confirm_transfer(Origin::signed(V1()), sub_message_id));
        // assert_ok!(Bridge::confirm_transfer(Origin::signed(USER1()), sub_message_id));
        //BurnedMessage(Hash, AccountId, H160, u64) event emitted

        assert_eq!(Balances::free_balance(USER2()), balance_of_user2 - amount2);
        assert_eq!(Balances::total_issuance(), total_issuance - amount2);
        assert_noop!(
            Bridge::cancel_transfer(Origin::signed(V2()), sub_message_id),
            "Failed to cancel. This transfer is already executed."
        );
    })
}

#[test]
fn update_validator_list_should_work() {
    new_test_ext().execute_with(|| {
        let eth_message_id = H256::from(ETH_MESSAGE_ID);
        const QUORUM: u64 = 3;

        assert_ok!(Bridge::update_validator_list(
            Origin::signed(V2()),
            eth_message_id,
            QUORUM,
            vec![V1(), V2(), V3(), V4()]
        ));
        let id = Bridge::message_id_by_transfer_id(0);
        let mut message = Bridge::validator_history(id);
        assert_eq!(message.status, Status::Pending);

        assert_ok!(Bridge::update_validator_list(
            Origin::signed(V1()),
            eth_message_id,
            QUORUM,
            vec![V1(), V2(), V3(), V4()]
        ));
        message = Bridge::validator_history(id);
        assert_eq!(message.status, Status::Confirmed);
        assert_eq!(Bridge::validators_count(), 4);
    })
}

#[test]
fn pause_the_bridge_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bridge::pause_bridge(Origin::signed(V2())));

        assert_eq!(Bridge::bridge_transfers_count(), 1);
        assert_eq!(Bridge::bridge_is_operational(), true);
        let id = Bridge::message_id_by_transfer_id(0);
        let mut message = Bridge::bridge_messages(id);
        assert_eq!(message.status, Status::Pending);

        assert_ok!(Bridge::pause_bridge(Origin::signed(V1())));
        assert_eq!(Bridge::bridge_is_operational(), false);
        message = Bridge::bridge_messages(id);
        assert_eq!(message.status, Status::Confirmed);
    })
}

#[test]
fn extrinsics_restricted_should_fail() {
    new_test_ext().execute_with(|| {
        let eth_message_id = H256::from(ETH_MESSAGE_ID);
        let eth_address = H160::from(ETH_ADDRESS);

        assert_ok!(Bridge::pause_bridge(Origin::signed(V2())));
        assert_ok!(Bridge::pause_bridge(Origin::signed(V1())));

        // substrate <-- Ethereum
        assert_noop!(
            Bridge::multi_signed_mint(Origin::signed(V2()), eth_message_id, eth_address, USER2(), 1000),
            "Bridge is not operational"
        );
    })
}

#[test]
fn double_pause_should_fail() {
    new_test_ext().execute_with(|| {
        assert_eq!(Bridge::bridge_is_operational(), true);
        assert_ok!(Bridge::pause_bridge(Origin::signed(V2())));
        assert_ok!(Bridge::pause_bridge(Origin::signed(V1())));
        assert_eq!(Bridge::bridge_is_operational(), false);
        assert_noop!(
            Bridge::pause_bridge(Origin::signed(V1())),
            "Bridge is not operational already"
        );
    })
}
#[test]
fn pause_and_resume_the_bridge_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Bridge::bridge_is_operational(), true);
        assert_ok!(Bridge::pause_bridge(Origin::signed(V2())));
        assert_ok!(Bridge::pause_bridge(Origin::signed(V1())));
        assert_eq!(Bridge::bridge_is_operational(), false);
        assert_ok!(Bridge::resume_bridge(Origin::signed(V1())));
        assert_ok!(Bridge::resume_bridge(Origin::signed(V2())));
        assert_eq!(Bridge::bridge_is_operational(), true);
    })
}

#[test]
fn double_vote_should_fail() {
    new_test_ext().execute_with(|| {
        assert_eq!(Bridge::bridge_is_operational(), true);
        assert_ok!(Bridge::pause_bridge(Origin::signed(V2())));
        assert_noop!(
            Bridge::pause_bridge(Origin::signed(V2())),
            "This validator has already voted."
        );
    })
}

#[test]
fn instant_withdraw_should_fail() {
    new_test_ext().execute_with(|| {
        let eth_message_id = H256::from(ETH_MESSAGE_ID);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 99;
        let amount2 = 49;

        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id,
            eth_address,
            USER3(),
            amount1
        ));
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V1()),
            eth_message_id,
            eth_address,
            USER3(),
            amount1
        ));
        //substrate ----> ETH
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER3()),
            eth_address,
            amount2
        ));
        //RelayMessage(message_id) event emitted
        let sub_message_id = Bridge::message_id_by_transfer_id(1);
        let get_message = || Bridge::messages(sub_message_id);
        let mut message = get_message();
        assert_eq!(message.status, Status::Withdraw);
        //approval
        assert_eq!(Balances::reserved_balance(USER3()), 0);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_eq!(
            Bridge::approve_transfer(Origin::signed(V2()), sub_message_id),
            Err(DispatchErrorWithPostInfo::from(
                "Cannot withdraw more that 75% of first day deposit."
            ))
        );

        message = get_message();
        assert_eq!(message.status, Status::Canceled);
    })
}

#[test]
fn change_limits_should_work() {
    new_test_ext().execute_with(|| {
        let max_tx_value = 10;
        let day_max_limit = 20;
        let day_max_limit_for_one_address = 5;
        let max_pending_tx_limit = 40;
        let min_tx_value = 1;

        assert_eq!(Bridge::current_limits().max_tx_value, 100);
        assert_ok!(Bridge::update_limits(
            Origin::signed(V2()),
            max_tx_value,
            day_max_limit,
            day_max_limit_for_one_address,
            max_pending_tx_limit,
            min_tx_value,
        ));
        assert_ok!(Bridge::update_limits(
            Origin::signed(V1()),
            max_tx_value,
            day_max_limit,
            day_max_limit_for_one_address,
            max_pending_tx_limit,
            min_tx_value,
        ));

        assert_eq!(Bridge::current_limits().max_tx_value, 10);
    })
}
#[test]
fn change_limits_should_fail() {
    new_test_ext().execute_with(|| {
        let day_max_limit = 20;
        let day_max_limit_for_one_address = 5;
        let max_pending_tx_limit = 40;
        let min_tx_value = 1;
        const MORE_THAN_MAX: u128 = u128::max_value();

        assert_noop!(
            Bridge::update_limits(
                Origin::signed(V1()),
                MORE_THAN_MAX,
                day_max_limit,
                day_max_limit_for_one_address,
                max_pending_tx_limit,
                min_tx_value,
            ),
            "Overflow setting limit"
        );
    })
}

#[test]
fn pending_burn_limit_should_work() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 60;
        let amount2 = 49;
        //TODO: pending transactions volume never reached if daily limit is lower
        // USER1(), USER2() init in mock.rs
        let _ = Balances::transfer(Origin::signed(USER1()), USER3(), amount1);
        let _ = Balances::transfer(Origin::signed(USER1()), USER4(), amount1);
        let _ = Balances::transfer(Origin::signed(USER1()), USER5(), amount1);
        let _ = Balances::transfer(Origin::signed(USER1()), USER6(), amount1);
        let _ = Balances::transfer(Origin::signed(USER1()), USER7(), amount1);
        let _ = Balances::transfer(Origin::signed(USER1()), USER8(), amount1);
        let _ = Balances::transfer(Origin::signed(USER1()), USER9(), amount1);
        //1
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER3()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(1);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER4()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(2);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER5()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(3);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER6()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(4);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER7()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(5);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER8()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(6);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER9()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(7);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));

        assert_eq!(Bridge::pending_burn_count(), amount2 * 8);
        assert_noop!(
            Bridge::set_transfer(Origin::signed(USER1()), eth_address, amount2),
            "Too many pending burn transactions."
        );
    })
}

#[test]
fn pending_mint_limit_should_work() {
    new_test_ext().execute_with(|| {
        let eth_message_id = H256::from(ETH_MESSAGE_ID);
        let eth_message_id1 = H256::from(ETH_MESSAGE_ID1);
        let eth_message_id2 = H256::from(ETH_MESSAGE_ID2);
        let eth_message_id3 = H256::from(ETH_MESSAGE_ID3);
        let eth_message_id4 = H256::from(ETH_MESSAGE_ID4);
        let eth_message_id5 = H256::from(ETH_MESSAGE_ID5);
        let eth_message_id6 = H256::from(ETH_MESSAGE_ID6);
        let eth_message_id7 = H256::from(ETH_MESSAGE_ID7);
        let eth_message_id8 = H256::from(ETH_MESSAGE_ID8);
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 49;

        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id,
            eth_address,
            USER2(),
            amount1
        ));

        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id2,
            eth_address,
            USER3(),
            amount1
        ));

        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id3,
            eth_address,
            USER4(),
            amount1
        ));

        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id4,
            eth_address,
            USER5(),
            amount1
        ));
        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id5,
            eth_address,
            USER6(),
            amount1
        ));
        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id6,
            eth_address,
            USER7(),
            amount1
        ));
        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id7,
            eth_address,
            USER8(),
            amount1
        ));
        //substrate <----- ETH
        assert_ok!(Bridge::multi_signed_mint(
            Origin::signed(V2()),
            eth_message_id8,
            eth_address,
            USER9(),
            amount1
        ));
        assert_eq!(Bridge::pending_mint_count(), amount1 * 8);

        //substrate <----- ETH
        assert_noop!(
            Bridge::multi_signed_mint(
                Origin::signed(V2()),
                eth_message_id1,
                eth_address,
                USER1(),
                amount1 + 5
            ),
            "Too many pending mint transactions."
        );
    })
}

#[test]
fn blocking_account_by_volume_should_work() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount1 = 9;
        let amount2 = 49;
        let amount3 = 101;

        assert_eq!(
            Bridge::set_transfer(Origin::signed(USER2()), eth_address, amount1),
            Err(DispatchErrorWithPostInfo::from(
                "Invalid amount for transaction. Reached minimum limit."
            ))
        );

        assert_eq!(
            Bridge::set_transfer(Origin::signed(USER2()), eth_address, amount3),
            Err(DispatchErrorWithPostInfo::from(
                "Invalid amount for transaction. Reached maximum limit."
            ))
        );

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));
        assert_ok!(Bridge::approve_transfer(Origin::signed(V2()), sub_message_id));

        assert_eq!(
            Bridge::set_transfer(Origin::signed(USER2()), eth_address, amount2),
            Err(DispatchErrorWithPostInfo::from(
                "Transfer declined, user blocked due to daily volume limit."
            ))
        );
    })
}

#[test]
fn blocked_account_unblocked_next_day_should_work() {
    new_test_ext().execute_with(|| {
        let eth_address = H160::from(ETH_ADDRESS);
        let amount2 = 49;
        run_to_block(DAY_IN_BLOCKS.into());

        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));
        let sub_message_id = Bridge::message_id_by_transfer_id(0);
        assert_ok!(Bridge::approve_transfer(Origin::signed(V1()), sub_message_id));
        assert_ok!(Bridge::approve_transfer(Origin::signed(V2()), sub_message_id));
        assert_eq!(
            Bridge::set_transfer(Origin::signed(USER2()), eth_address, amount2),
            Err(DispatchErrorWithPostInfo::from(
                "Transfer declined, user blocked due to daily volume limit."
            ))
        );

        //user added to blocked vec
        let blocked_vec: Vec<sr25519::Public> = vec![USER2()];
        assert_eq!(Bridge::daily_blocked(1), blocked_vec);

        run_to_block((DAY_IN_BLOCKS * 2).into());
        run_to_block((DAY_IN_BLOCKS * 3).into());

        //try again
        assert_ok!(Bridge::set_transfer(
            Origin::signed(USER2()),
            eth_address,
            amount2
        ));
    })
}
