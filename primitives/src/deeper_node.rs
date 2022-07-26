use sp_core::H160;

pub trait NodeInterface<AccountId, BlockNumber> {
    /// This function tells if the device has been offline for a day
    fn get_onboard_time(account_id: &AccountId) -> Option<BlockNumber>;

    /// This function tells if the device has ever been online
    fn im_ever_online(account_id: &AccountId) -> bool;

    /// This function returns how many eras the device has been offline
    fn get_eras_offline(account_id: &AccountId) -> u32;

    /// This function returns evm address associated with account
    fn get_accounts_deeper_evm(account_id: &AccountId) -> Option<H160>;

    /// This function returns deeper address associated with evm account
    fn get_accounts_evm_deeper(evm_address: &H160) -> Option<AccountId>;
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

    fn get_accounts_deeper_evm(_account_id: &AccountId) -> Option<H160> {
        None
    }

    fn get_accounts_evm_deeper(_evm_address: &H160) -> Option<AccountId> {
        None
    }
}
