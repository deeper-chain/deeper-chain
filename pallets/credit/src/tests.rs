use crate::{mock::*, Error};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    weights::PostDispatchInfo,
};
use sp_core::sr25519::{Public, Signature};
use sp_io::crypto::sr25519_verify;

#[test]
fn fn_initialize_credit_extrinsic() {
    new_test_ext().execute_with(|| {
        // initialize credit [0-29]
        assert_ok!(Credit::initialize_credit(Origin::signed(1)));
        assert_eq!(Credit::get_user_credit(1), Some(60));
    });
}
