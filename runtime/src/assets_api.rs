use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait AssetsApi<AccountId, AssetBalance, AssetId>
    where
        AccountId: Codec,
        AssetBalance: Codec,
        AssetId: Codec,
    {
        /// Returns the list of `AssetId`s and corresponding balance that an `AccountId` has.
        fn account_balances(account: AccountId) -> Vec<(AssetId, AssetBalance)>;
    }
}
