use crate::{mock::*, Error, NodeInterface};
use frame_support::{assert_ok, dispatch::DispatchErrorWithPostInfo};

#[test]
fn fn_register_device() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
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
            Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidIP))
        );

        // register device with invalid country code
        assert_eq!(
            DeeperNode::register_device(
                Origin::signed(3),
                vec![1, 2, 3, 4],
                "ZZ".as_bytes().to_vec()
            ),
            Err(DispatchErrorWithPostInfo::from(Error::<Test>::InvalidCode))
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
        DeeperNode::setup_region_map();
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
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );
    });
}

#[test]
fn fn_register_server() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
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
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );

        // register server with invalid duration
        assert_ok!(DeeperNode::register_device(
            Origin::signed(3),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_eq!(
            DeeperNode::register_server(Origin::signed(3), 8),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DurationOverflow
            ))
        );
    });
}

#[test]
fn fn_unregister_server() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
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
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );
    });
}

#[test]
fn fn_update_server() {
    new_test_ext().execute_with(|| {
        DeeperNode::setup_region_map();
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
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DeviceNotRegister
            ))
        );

        // register device, then register server
        assert_ok!(DeeperNode::register_device(
            Origin::signed(3),
            vec![1, 2, 3, 4],
            "US".as_bytes().to_vec()
        ));
        assert_eq!(
            DeeperNode::update_server(Origin::signed(3), 10),
            Err(DispatchErrorWithPostInfo::from(
                Error::<Test>::DurationOverflow
            ))
        );
    });
}

#[test]
fn im_online() {
    new_test_ext().execute_with(|| {
        assert_eq!(DeeperNode::get_onboard_time(1), None);
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(DeeperNode::get_im_online(1), Some(0));
        assert_eq!(DeeperNode::get_onboard_time(1), Some(0));
        run_to_block(1);
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(DeeperNode::get_im_online(1), Some(1));
        assert_eq!(DeeperNode::get_onboard_time(1), Some(0));
    });
}

#[test]
fn im_offline() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        run_to_block(1);
        assert_eq!(DeeperNode::im_offline(&1), false);
        run_to_block(24 * 3600 * 1000 / 5000);
        assert_eq!(DeeperNode::im_offline(&1), true);
    });
}

#[test]
fn im_ever_online() {
    new_test_ext().execute_with(|| {
        assert_eq!(DeeperNode::im_ever_online(&1), false);
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        assert_eq!(DeeperNode::im_ever_online(&1), true);
    });
}

#[test]
fn get_days_offline() {
    new_test_ext().execute_with(|| {
        assert_ok!(DeeperNode::im_online(Origin::signed(1)));
        run_to_block(24 * 3600 * 1000 / 5000 - 1);
        assert_eq!(DeeperNode::get_days_offline(&1), 0);
        run_to_block(24 * 3600 * 1000 / 5000);
        assert_eq!(DeeperNode::get_days_offline(&1), 1);
        run_to_block(3 * 24 * 3600 * 1000 / 5000);
        assert_eq!(DeeperNode::get_days_offline(&1), 3);
        assert_eq!(DeeperNode::get_days_offline(&2), 3);
    });
}
