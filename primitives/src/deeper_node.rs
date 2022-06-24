pub trait NodeInterface<AccountId, BlockNumber> {
    /// This function tells if the device has been offline for a day
    fn get_onboard_time(account_id: &AccountId) -> Option<BlockNumber>;

    /// This function tells if the device has ever been online
    fn im_ever_online(account_id: &AccountId) -> bool;

    /// This function returns how many eras the device has been offline
    fn get_eras_offline(account_id: &AccountId) -> u32;
}

impl<AccountId, BlockNumber> NodeInterface<AccountId, BlockNumber> for () {
    fn get_onboard_time(_account_id: &AccountId) -> Option<BlockNumber> {
        None
    }

    fn im_ever_online(_account_id: &AccountId) -> bool {
        true
    }

    fn get_eras_offline(_account_id: &AccountId) -> u32 {
        0
    }
}
