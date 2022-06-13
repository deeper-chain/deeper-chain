// Copyright (C) 2021 Deeper Network Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

pub mod weights;
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::storage::migration::get_storage_value;
    use frame_support::{ensure, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_system::{self, ensure_signed};

    use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
    use enumflags2::{bitflags, BitFlags};
    use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
    pub use sp_core::H160;
    use sp_runtime::{traits::StaticLookup, RuntimeDebug};

    pub trait UserPrivilegeInterface<Account> {
        fn has_privilege(user: &Account, p: Privilege) -> bool;
        fn has_evm_privilege(user: &H160, p: Privilege) -> bool;
    }

    #[bitflags]
    #[repr(u64)]
    #[derive(Clone, Copy, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo)]
    pub enum Privilege {
        LockerMember = 1 << 0,
        ReleaseSetter = 1 << 1,
        EvmAddressSetter = 1 << 2,
        EvmCreditOperation = 1 << 3,
        NpowMint = 1 << 4,
    }

    /// Wrapper type for `BitFlags<Privilege>` that implements `Codec`.
    #[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
    pub struct Privileges(pub BitFlags<Privilege>);

    impl MaxEncodedLen for Privileges {
        fn max_encoded_len() -> usize {
            u64::max_encoded_len()
        }
    }

    impl Eq for Privileges {}
    impl Encode for Privileges {
        fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
            self.0.bits().using_encoded(f)
        }
    }

    impl EncodeLike for Privileges {}

    impl Decode for Privileges {
        fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
            let field = u64::decode(input)?;
            Ok(Self(
                <BitFlags<Privilege>>::from_bits(field as u64).map_err(|_| "invalid value")?,
            ))
        }
    }
    impl TypeInfo for Privileges {
        type Identity = Self;

        fn type_info() -> Type {
            Type::builder()
                .path(Path::new("BitFlags", module_path!()))
                .type_params(sp_std::vec![TypeParameter::new(
                    "T",
                    Some(meta_type::<Privilege>()),
                )])
                .composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("Privilege")))
        }
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum Releases {
        V1_0_0,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type ForceOrigin: EnsureOrigin<Self::Origin>;
        type WeightInfo: WeightInfo;
        type UserPrivilegeInterface: UserPrivilegeInterface<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    //#[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        UserPrivilegeSet(T::AccountId, Privilege),
        UserPrivilegeUnSet(T::AccountId, Privilege),
        UserPrivilegeClear(T::AccountId),
        EvmPrivilegeSet(H160, Privilege),
        EvmPrivilegeUnSet(H160, Privilege),
        EvmPrivilegeClear(H160),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// not privilege flag
        NotExistPrivilege,
        /// not has right to do
        NoPermission,
    }

    #[pallet::storage]
    #[pallet::getter(fn user_privileges)]
    pub(super) type UserPrivileges<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Privileges, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn evm_address_privileges)]
    pub(super) type EvmAddressPrivileges<T: Config> =
        StorageMap<_, Twox64Concat, H160, Privileges, OptionQuery>;

    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Releases>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            use frame_support::traits::ConstU32;
            use frame_support::WeakBoundedVec;

            if StorageVersion::<T>::get().is_none() {
                let lockers = get_storage_value::<WeakBoundedVec<T::AccountId, ConstU32<50>>>(
                    b"Operation",
                    b"LockMemberWhiteList",
                    &[],
                );
                if let Some(lockers) = lockers {
                    let lockers = lockers.into_inner();
                    for locker in lockers {
                        UserPrivileges::<T>::insert(
                            locker,
                            Privileges(Privilege::LockerMember.into()),
                        );
                    }
                }
                let setter =
                    get_storage_value::<T::AccountId>(b"Operation", b"ReleasePaymentAddress", &[]);
                if let Some(setter) = setter {
                    UserPrivileges::<T>::insert(
                        setter,
                        Privileges(Privilege::ReleaseSetter.into()),
                    );
                }

                StorageVersion::<T>::put(Releases::V1_0_0);
                return T::DbWeight::get().reads_writes(1, 1);
            }
            0
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::set_user_privilege())]
        pub fn set_user_privilege(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            privilege: Privilege,
        ) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;
            let who = T::Lookup::lookup(who)?;

            let old_priv = Self::user_privileges(&who);
            let new_priv = {
                match old_priv {
                    Some(old_priv) => old_priv.0 | BitFlags::from_flag(privilege),
                    None => privilege.into(),
                }
            };

            UserPrivileges::<T>::insert(&who, Privileges(new_priv));
            Self::deposit_event(Event::UserPrivilegeSet(who, privilege));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_user_privilege())]
        pub fn unset_user_privilege(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
            privilege: Privilege,
        ) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;
            let who = T::Lookup::lookup(who)?;

            let old_priv = Self::user_privileges(&who);
            if old_priv.is_none() {
                return Err(Error::<T>::NotExistPrivilege.into());
            }
            let mut new_priv = old_priv.unwrap();
            new_priv.0.remove(privilege);
            UserPrivileges::<T>::insert(&who, new_priv);
            Self::deposit_event(Event::UserPrivilegeUnSet(who, privilege));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::clear_user_privilege())]
        pub fn clear_user_privilege(
            origin: OriginFor<T>,
            who: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResult {
            T::ForceOrigin::ensure_origin(origin)?;
            let who = T::Lookup::lookup(who)?;
            UserPrivileges::<T>::remove(&who);
            Self::deposit_event(Event::UserPrivilegeClear(who));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_evm_privilege())]
        pub fn set_evm_privilege(
            origin: OriginFor<T>,
            who: H160,
            privilege: Privilege,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&sender, Privilege::EvmAddressSetter),
                Error::<T>::NoPermission
            );
            let old_priv = Self::evm_address_privileges(&who);
            let new_priv = {
                match old_priv {
                    Some(old_priv) => old_priv.0 | BitFlags::from_flag(privilege),
                    None => privilege.into(),
                }
            };
            EvmAddressPrivileges::<T>::insert(&who, Privileges(new_priv));
            Self::deposit_event(Event::EvmPrivilegeSet(who, privilege));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::set_evm_privilege())]
        pub fn unset_evm_privilege(
            origin: OriginFor<T>,
            who: H160,
            privilege: Privilege,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&sender, Privilege::EvmAddressSetter),
                Error::<T>::NoPermission
            );
            let old_priv = Self::evm_address_privileges(&who);
            if old_priv.is_none() {
                return Err(Error::<T>::NotExistPrivilege.into());
            }
            let mut new_priv = old_priv.unwrap();
            new_priv.0.remove(privilege);
            EvmAddressPrivileges::<T>::insert(&who, new_priv);

            Self::deposit_event(Event::EvmPrivilegeUnSet(who, privilege));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::clear_evm_privilege())]
        pub fn clear_evm_privilege(origin: OriginFor<T>, who: H160) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                T::UserPrivilegeInterface::has_privilege(&sender, Privilege::EvmAddressSetter),
                Error::<T>::NoPermission
            );
            EvmAddressPrivileges::<T>::remove(&who);
            Self::deposit_event(Event::EvmPrivilegeClear(who));
            Ok(().into())
        }
    }

    pub struct DefaultPrivilegeHandler<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> UserPrivilegeInterface<T::AccountId> for DefaultPrivilegeHandler<T> {
        fn has_privilege(user: &T::AccountId, p: Privilege) -> bool {
            let privs = UserPrivileges::<T>::get(user);
            match privs {
                None => false,
                Some(privs) => privs.0.contains(p),
            }
        }

        fn has_evm_privilege(user: &H160, p: Privilege) -> bool {
            let privs = EvmAddressPrivileges::<T>::get(user);
            match privs {
                None => false,
                Some(privs) => privs.0.contains(p),
            }
        }
    }

    impl<T: Config> UserPrivilegeInterface<T::AccountId> for Pallet<T> {
        fn has_privilege(user: &T::AccountId, p: Privilege) -> bool {
            T::UserPrivilegeInterface::has_privilege(user, p)
        }

        fn has_evm_privilege(user: &H160, p: Privilege) -> bool {
            T::UserPrivilegeInterface::has_evm_privilege(user, p)
        }
    }
}
