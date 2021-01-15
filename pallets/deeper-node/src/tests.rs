use crate::mock::*;
use frame_support::assert_ok;
//use frame_system::ensure_signed;
use sp_runtime::DispatchError;

#[test]
fn fn_register_device() {
    new_test_ext().execute_with(|| {
        // register device
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        let node = DeeperNode::get_device_info(1);
        assert_eq!(node.ipv4, vec![1, 2, 3, 4]);
        assert_eq!(node.country, "US".as_bytes().to_vec());

        // register device with invalid ip (length > 256)
        assert_eq!(
            DeeperNode::register_device(Origin::signed(2), vec![1; 257], "US".as_bytes().to_vec()),
            Err(DispatchError::Other("invalid ip address",))
        );

        // register device with invalid country code
        assert_eq!(
            DeeperNode::register_device(
                Origin::signed(3),
                vec![1, 2, 3, 4],
                "ZZ".as_bytes().to_vec()
            ),
            Err(DispatchError::Other("invalid country code",))
        );

        // register device twice
        assert_ok!(DeeperNode::register_device(
            Origin::signed(4),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::register_device(
            Origin::signed(4),
            vec![1, 3, 4, 5],
            "CA".as_bytes().to_vec()
        ));
        let node = DeeperNode::get_device_info(4);
        assert_eq!(node.ipv4, vec![1, 3, 4, 5]);
        assert_eq!(node.country, "CA".as_bytes().to_vec());
    });
}

#[test]
fn fn_unregister_device() {
    new_test_ext().execute_with(|| {
        // unregister a registered device
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::unregister_device(Origin::signed(1)));

        // unregister an unregistered device
        assert_eq!(
            DeeperNode::unregister_device(Origin::signed(2)),
            Err(DispatchError::Other("device not registered!",))
        );
    });
}

#[test]
fn fn_register_server() {
    new_test_ext().execute_with(|| {
        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::register_server(Origin::signed(1), 1));
        let servers = DeeperNode::get_servers_by_country("US".as_bytes().to_vec());
        let index = servers.iter().position(|x| *x == 1);
        assert_eq!(index, Some(0));

        // register server before register device
        assert_eq!(
            DeeperNode::register_server(Origin::signed(2), 1),
            Err(DispatchError::Other("sender device needs register first",))
        );

        // register server with invalid duration
        assert_ok!(DeeperNode::register_device(
            Origin::signed(3),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_eq!(
            DeeperNode::register_server(Origin::signed(3), 8),
            Err(DispatchError::Other("duration is too big",))
        );
    });
}

#[test]
fn fn_unregister_server() {
    new_test_ext().execute_with(|| {
        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::register_server(Origin::signed(1), 1));
        assert_ok!(DeeperNode::unregister_server(Origin::signed(1)));

        // register server before register device
        assert_eq!(
            DeeperNode::unregister_server(Origin::signed(2)),
            Err(DispatchError::Other("sender device needs register first",))
        );
    });
}

#[test]
fn fn_update_server() {
    new_test_ext().execute_with(|| {
        // register device, then update server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(1),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_ok!(DeeperNode::update_server(Origin::signed(1), 1));

        // update server before register device
        assert_eq!(
            DeeperNode::update_server(Origin::signed(2), 1),
            Err(DispatchError::Other("sender device needs register first",))
        );

        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(3),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_eq!(
            DeeperNode::update_server(Origin::signed(3), 10),
            Err(DispatchError::Other("duration is too big",))
        );
    });
}