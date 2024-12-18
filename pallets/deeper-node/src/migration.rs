use super::*;
pub mod v1 {
    use codec::{Decode, Encode};
    use frame_support::{pallet_prelude::*, weights::Weight, LOG_TARGET};
    use scale_info::TypeInfo;

    use super::*;
    use frame_support::traits::OnRuntimeUpgrade;
    use frame_system::pallet_prelude::BlockNumberFor;
    // use crate::{CountryRegion,Config,Pallet};
    pub(crate) type IpV4 = Vec<u8>;

    // struct to store the registered Device Information
    #[derive(Decode, Encode, TypeInfo)]
    pub struct OldNode<AccountId, BlockNumber> {
        pub account_id: AccountId,
        ipv4: IpV4,
        country: CountryRegion,
        expire: BlockNumber,
    }

    pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
        fn on_runtime_upgrade() -> Weight {
            let current_version = Pallet::<T>::current_storage_version();
            let onchain_version = Pallet::<T>::on_chain_storage_version();
            if onchain_version == 0 && current_version == 1 {
                let mut translated = 0u64;
                DeviceInfo::<T>::translate(
                    |_key, old_node: OldNode<T::AccountId, BlockNumberFor<T>>| {
                        translated += 1;
                        let node = Node {
                            account_id: old_node.account_id.clone(),
                            country: old_node.country.clone(),
                            expire: old_node.expire,
                        };
                        Some(node)
                    },
                );

                current_version.put::<Pallet<T>>();
                log::info!(
                    target: LOG_TARGET,
                    "Upgraded {} node, storage to version {:?}",
                    translated,
                    current_version
                );
                T::DbWeight::get().reads_writes(translated + 1, translated + 1)
            } else {
                log::info!(
                    target: LOG_TARGET,
                    "Migration did not execute. This probably should be removed"
                );
                T::DbWeight::get().reads(1)
            }
        }
    }
}
