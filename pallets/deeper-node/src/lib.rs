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

use frame_support::codec::{Decode, Encode};
use scale_info::TypeInfo;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
pub mod weights;
use sp_std::prelude::*;
pub use weights::WeightInfo;

pub type IpV4 = Vec<u8>;
pub type CountryRegion = Vec<u8>;
pub type DurationEras = u8;

// struct to store the registered Device Information
#[derive(Decode, Encode, Default, TypeInfo)]
pub struct Node<AccountId, BlockNumber> {
    pub account_id: AccountId,
    ipv4: IpV4, // IP will not be exposed in future version
    country: CountryRegion,
    expire: BlockNumber,
}

pub trait NodeInterface<AccountId, BlockNumber> {
    /// This function tells if the device has been offline for a day
    fn get_onboard_time(account_id: &AccountId) -> Option<BlockNumber>;

    /// This function tells if the device has ever been online
    fn im_ever_online(account_id: &AccountId) -> bool;

    /// This function returns how many eras the device has been offline
    fn get_eras_offline(account_id: &AccountId) -> u32;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::{Currency, Get, ReservableCurrency};
    use frame_support::{dispatch::DispatchResult, ensure};
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use frame_system::{self, ensure_signed};
    use sp_std::convert::TryInto;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type MinLockAmt: Get<u32>;
        type MaxDurationEras: Get<u8>;
        /// Number of blocks per era.
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        type MaxIpLength: Get<usize>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn map_init)]
    pub type RegionMapInit<T: Config> = StorageValue<_, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn region_code)]
    pub(super) type RegionMap<T: Config> =
        StorageMap<_, Blake2_128Concat, CountryRegion, CountryRegion, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn device_info)]
    pub(super) type DeviceInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Node<T::AccountId, T::BlockNumber>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn servers_by_country)]
    pub(super) type ServersByCountry<T: Config> =
        StorageMap<_, Blake2_128Concat, CountryRegion, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn servers_by_region)]
    pub(super) type ServersByRegion<T: Config> =
        StorageMap<_, Blake2_128Concat, CountryRegion, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_im_online)]
    pub(super) type ImOnline<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn onboard_time)]
    pub(super) type OnboardTime<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn devices_onboard)]
    pub(super) type DevicesOnboard<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tmp: BalanceOf<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                tmp: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::setup_region_map();
        }
    }

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // register node: AccountId, ipv4, country
        RegisterNode(T::AccountId, IpV4, CountryRegion),
        UnregisterNode(T::AccountId),

        // add account into a country's server list
        ServerCountryAdded(T::AccountId, CountryRegion, T::BlockNumber, u64),
        // remove account from a country's server list
        ServerCountryRemoved(T::AccountId, CountryRegion),

        // add account into a region's server list
        ServerRegionAdded(T::AccountId, CountryRegion, T::BlockNumber, u64),
        // remove account from a region's server list
        ServerRegionRemoved(T::AccountId, CountryRegion),

        ImOnline(T::AccountId, T::BlockNumber),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// double country registration is not allowed
        DoubleCountryRegistration,
        /// double level 2 region registration is not allowed
        DoubleLevel2Registration,
        /// double level 3 region registration is not allowed
        DoubleLevel3Registration,
        /// invalid country or region code
        InvalidCode,
        /// invalid ip address
        InvalidIP,
        /// device is not registered
        DeviceNotRegister,
        /// channel duration is too large
        DurationOverflow,
        /// region map is not initialized
        InvalidRegionMap,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::register_device())]
        pub fn register_device(
            origin: OriginFor<T>,
            ip: IpV4,
            country: CountryRegion,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                <RegionMapInit<T>>::get() == true,
                Error::<T>::InvalidRegionMap
            );
            ensure!(
                <RegionMap<T>>::contains_key(&country),
                Error::<T>::InvalidCode
            );
            ensure!(ip.len() <= T::MaxIpLength::get(), Error::<T>::InvalidIP);

            if !<DeviceInfo<T>>::contains_key(&sender) {
                let node = Node {
                    account_id: sender.clone(),
                    ipv4: ip.clone(),
                    country: country.clone(),
                    expire: <frame_system::Pallet<T>>::block_number(),
                };
                T::Currency::reserve(&sender, BalanceOf::<T>::from(T::MinLockAmt::get()))?;
                <DeviceInfo<T>>::insert(&sender, node);
            } else {
                <DeviceInfo<T>>::mutate(&sender, |node| {
                    if node.country != country {
                        let _ = Self::try_remove_server(&sender);
                        node.country = country.clone();
                    }
                    node.ipv4 = ip.clone();
                    node.expire = <frame_system::Pallet<T>>::block_number();
                });
            }
            Self::deposit_event(Event::RegisterNode(sender, ip, country));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::unregister_device())]
        pub fn unregister_device(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                <DeviceInfo<T>>::contains_key(&sender),
                Error::<T>::DeviceNotRegister
            );
            let _ = Self::try_remove_server(&sender);
            <DeviceInfo<T>>::remove(&sender);
            T::Currency::unreserve(&sender, BalanceOf::<T>::from(T::MinLockAmt::get()));
            Self::deposit_event(Event::UnregisterNode(sender));
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::unregister_device())]
        pub fn register_server(
            origin: OriginFor<T>,
            duration_eras: DurationEras,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                <DeviceInfo<T>>::contains_key(&sender),
                Error::<T>::DeviceNotRegister
            );
            ensure!(
                duration_eras <= T::MaxDurationEras::get(),
                Error::<T>::DurationOverflow
            );
            let blocks = T::BlockNumber::from(duration_eras) * T::BlocksPerEra::get();
            let _ = Self::try_add_server(&sender, blocks);
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::update_server())]
        pub fn update_server(
            origin: OriginFor<T>,
            duration_eras: DurationEras,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                <DeviceInfo<T>>::contains_key(&sender),
                Error::<T>::DeviceNotRegister
            );
            ensure!(
                duration_eras <= T::MaxDurationEras::get(),
                Error::<T>::DurationOverflow
            );
            let blocks = T::BlockNumber::from(duration_eras) * T::BlocksPerEra::get();
            <DeviceInfo<T>>::mutate(&sender, |node| {
                node.expire = <frame_system::Pallet<T>>::block_number() + blocks;
            });
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::unregister_server())]
        pub fn unregister_server(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                <DeviceInfo<T>>::contains_key(&sender),
                Error::<T>::DeviceNotRegister
            );
            let _ = Self::try_remove_server(&sender);
            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::im_online())]
        pub fn im_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let current_block = <frame_system::Pallet<T>>::block_number();
            ImOnline::<T>::insert(&sender, current_block.clone());
            if !OnboardTime::<T>::contains_key(&sender) {
                OnboardTime::<T>::insert(&sender, current_block.clone());
                DevicesOnboard::<T>::mutate(|devices| devices.push(sender.clone()));
            }
            Self::deposit_event(Event::ImOnline(sender, current_block));
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        // try to remove an account from country and region server lists if exists
        fn try_remove_server(sender: &T::AccountId) -> DispatchResult {
            let mut node = <DeviceInfo<T>>::get(&sender);
            let first_region = <RegionMap<T>>::get(&node.country);
            let sec_region = <RegionMap<T>>::get(&first_region);

            // remove from country server list
            let mut server_list = <ServersByCountry<T>>::get(&node.country);
            let _ = Self::country_list_remove(&mut server_list, &sender, &node.country);

            // remove from level 3 region server list
            server_list = <ServersByRegion<T>>::get(&first_region);
            let _ = Self::region_list_remove(&mut server_list, &sender, &first_region);

            // remove from level 2 region server list
            server_list = <ServersByRegion<T>>::get(&sec_region);
            let _ = Self::region_list_remove(&mut server_list, &sender, &sec_region);

            // ensure consistency
            node.expire = <frame_system::Pallet<T>>::block_number();
            <DeviceInfo<T>>::insert(&sender, node);

            Ok(())
        }

        // try to add an account to a country's server list; no double add
        fn try_add_server(sender: &T::AccountId, duration: T::BlockNumber) -> DispatchResult {
            let mut node = <DeviceInfo<T>>::get(&sender);
            let first_region = <RegionMap<T>>::get(&node.country);
            let sec_region = <RegionMap<T>>::get(&first_region);

            // country registration
            let mut country_server_list = <ServersByCountry<T>>::get(&node.country);
            if Self::country_list_insert(
                &mut country_server_list,
                &sender,
                &node.country,
                &duration,
            ) == false
            {
                Err(Error::<T>::DoubleCountryRegistration)?
            }

            // level 3 region registration
            let mut level3_server_list = <ServersByRegion<T>>::get(&first_region);
            if Self::region_list_insert(&mut level3_server_list, &sender, &first_region, &duration)
                == false
            {
                let _ = Self::country_list_remove(&mut country_server_list, &sender, &node.country);
                Err(Error::<T>::DoubleLevel3Registration)?
            }

            // level 2 region registration
            let mut level2_server_list = <ServersByRegion<T>>::get(&sec_region);
            if Self::region_list_insert(&mut level2_server_list, &sender, &sec_region, &duration)
                == false
            {
                let _ = Self::country_list_remove(&mut country_server_list, &sender, &node.country);
                let _ = Self::region_list_remove(&mut level3_server_list, &sender, &first_region);
                Err(Error::<T>::DoubleLevel2Registration)?
            }

            // ensure consistency
            node.expire = <frame_system::Pallet<T>>::block_number() + duration;
            <DeviceInfo<T>>::insert(&sender, node);

            Ok(())
        }

        fn country_list_insert(
            servers: &mut Vec<T::AccountId>,
            account: &T::AccountId,
            country: &CountryRegion,
            duration: &T::BlockNumber,
        ) -> bool {
            match servers.binary_search(&account) {
                Ok(_) => false,
                Err(index) => {
                    servers.insert(index, account.clone());
                    <ServersByCountry<T>>::insert(&country, servers);
                    Self::deposit_event(Event::ServerCountryAdded(
                        account.clone(),
                        country.clone(),
                        duration.clone(),
                        index as u64,
                    ));
                    true
                }
            }
        }

        fn country_list_remove(
            servers: &mut Vec<T::AccountId>,
            account: &T::AccountId,
            country: &CountryRegion,
        ) -> bool {
            match servers.binary_search(&account) {
                Ok(index) => {
                    servers.remove(index);
                    <ServersByCountry<T>>::insert(&country, servers);
                    Self::deposit_event(Event::ServerCountryRemoved(
                        account.clone(),
                        country.clone(),
                    ));
                    true
                }
                Err(_) => false,
            }
        }

        fn region_list_insert(
            servers: &mut Vec<T::AccountId>,
            account: &T::AccountId,
            region: &CountryRegion,
            duration: &T::BlockNumber,
        ) -> bool {
            match servers.binary_search(&account) {
                Ok(_) => false,
                Err(index) => {
                    servers.insert(index, account.clone());
                    <ServersByRegion<T>>::insert(&region, servers);
                    Self::deposit_event(Event::ServerRegionAdded(
                        account.clone(),
                        region.clone(),
                        duration.clone(),
                        index as u64,
                    ));
                    true
                }
            }
        }

        fn region_list_remove(
            servers: &mut Vec<T::AccountId>,
            account: &T::AccountId,
            region: &CountryRegion,
        ) -> bool {
            match servers.binary_search(&account) {
                Ok(index) => {
                    servers.remove(index);
                    <ServersByRegion<T>>::insert(&region, servers);
                    Self::deposit_event(Event::ServerRegionRemoved(
                        account.clone(),
                        region.clone(),
                    ));
                    true
                }
                Err(_) => false,
            }
        }

        pub fn setup_region_map() {
            /* level 1 */
            /*
            RegionMap::insert("AMER".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
            RegionMap::insert("ASIA".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
            RegionMap::insert("AFRI".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
            RegionMap::insert("EURO".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
            RegionMap::insert("OCEA".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
            */

            /* level 2 */
            <RegionMap<T>>::insert("AMN".as_bytes().to_vec(), "AMER".as_bytes().to_vec());
            <RegionMap<T>>::insert("AMC".as_bytes().to_vec(), "AMER".as_bytes().to_vec());
            <RegionMap<T>>::insert("AMM".as_bytes().to_vec(), "AMER".as_bytes().to_vec());
            <RegionMap<T>>::insert("AMS".as_bytes().to_vec(), "AMER".as_bytes().to_vec());

            <RegionMap<T>>::insert("ASC".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
            <RegionMap<T>>::insert("ASE".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
            <RegionMap<T>>::insert("ASW".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
            <RegionMap<T>>::insert("ASS".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
            <RegionMap<T>>::insert("ASD".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());

            <RegionMap<T>>::insert("AFN".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
            <RegionMap<T>>::insert("AFM".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
            <RegionMap<T>>::insert("AFE".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
            <RegionMap<T>>::insert("AFW".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
            <RegionMap<T>>::insert("AFS".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());

            <RegionMap<T>>::insert("EUN".as_bytes().to_vec(), "EURO".as_bytes().to_vec());
            <RegionMap<T>>::insert("EUE".as_bytes().to_vec(), "EURO".as_bytes().to_vec());
            <RegionMap<T>>::insert("EUW".as_bytes().to_vec(), "EURO".as_bytes().to_vec());
            <RegionMap<T>>::insert("EUS".as_bytes().to_vec(), "EURO".as_bytes().to_vec());

            <RegionMap<T>>::insert("OCP".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());
            <RegionMap<T>>::insert("OCA".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());
            <RegionMap<T>>::insert("OCM".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());
            <RegionMap<T>>::insert("OCN".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());

            /* level 3 */
            <RegionMap<T>>::insert("BM".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
            <RegionMap<T>>::insert("CA".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
            <RegionMap<T>>::insert("GL".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
            <RegionMap<T>>::insert("PM".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
            <RegionMap<T>>::insert("US".as_bytes().to_vec(), "AMN".as_bytes().to_vec());

            <RegionMap<T>>::insert("AG".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("AI".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("AW".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("BB".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("BL".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("BQ".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("BS".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("CU".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("CW".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("DM".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("DO".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("GD".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("GP".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("HT".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("JM".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("KN".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("KY".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("LC".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("MF".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("MQ".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("MS".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("PR".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("SX".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("TC".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("TT".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("VC".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("VG".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
            <RegionMap<T>>::insert("VI".as_bytes().to_vec(), "AMC".as_bytes().to_vec());

            <RegionMap<T>>::insert("BZ".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("CR".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("GT".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("HN".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("MX".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("NI".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("PA".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
            <RegionMap<T>>::insert("SV".as_bytes().to_vec(), "AMM".as_bytes().to_vec());

            <RegionMap<T>>::insert("AR".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("BO".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("BR".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("CL".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("CO".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("EC".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("FK".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("GF".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("GS".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("GY".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("PE".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("PY".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("SR".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("UY".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
            <RegionMap<T>>::insert("VE".as_bytes().to_vec(), "AMS".as_bytes().to_vec());

            <RegionMap<T>>::insert("KG".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
            <RegionMap<T>>::insert("KZ".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
            <RegionMap<T>>::insert("TJ".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
            <RegionMap<T>>::insert("TM".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
            <RegionMap<T>>::insert("UZ".as_bytes().to_vec(), "ASC".as_bytes().to_vec());

            <RegionMap<T>>::insert("CN".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("HK".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("JP".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("KP".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("KR".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MN".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MO".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
            <RegionMap<T>>::insert("TW".as_bytes().to_vec(), "ASE".as_bytes().to_vec());

            <RegionMap<T>>::insert("AE".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("AM".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("AZ".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("BH".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("CY".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("GE".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("IL".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("IQ".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("JO".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("KW".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("LB".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("OM".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("PS".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("QA".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("SA".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("SY".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("TR".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
            <RegionMap<T>>::insert("YE".as_bytes().to_vec(), "ASW".as_bytes().to_vec());

            <RegionMap<T>>::insert("AF".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("BD".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("BT".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("IN".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("IR".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("LK".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("MV".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("NP".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
            <RegionMap<T>>::insert("PK".as_bytes().to_vec(), "ASS".as_bytes().to_vec());

            <RegionMap<T>>::insert("BN".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("ID".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("KH".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("LA".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("MM".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("MY".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("PH".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("SG".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("TH".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("TL".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
            <RegionMap<T>>::insert("VN".as_bytes().to_vec(), "ASD".as_bytes().to_vec());

            <RegionMap<T>>::insert("DZ".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
            <RegionMap<T>>::insert("EG".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
            <RegionMap<T>>::insert("LY".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
            <RegionMap<T>>::insert("MA".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
            <RegionMap<T>>::insert("SD".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
            <RegionMap<T>>::insert("TN".as_bytes().to_vec(), "AFN".as_bytes().to_vec());

            <RegionMap<T>>::insert("AO".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("CD".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("CF".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("CG".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("CM".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("GA".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("GQ".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("ST".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
            <RegionMap<T>>::insert("TD".as_bytes().to_vec(), "AFM".as_bytes().to_vec());

            <RegionMap<T>>::insert("BI".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("DJ".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("ER".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("ET".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("IO".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("KE".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("KM".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MG".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MU".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MW".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MZ".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("RE".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("RW".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("SC".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("SO".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("SS".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("TF".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("TZ".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("UG".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("YT".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("ZM".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
            <RegionMap<T>>::insert("ZW".as_bytes().to_vec(), "AFE".as_bytes().to_vec());

            <RegionMap<T>>::insert("BF".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("BJ".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("CI".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("CV".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("GH".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("GM".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("GN".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("GW".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("LR".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("ML".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("MR".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("NE".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("NG".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("SH".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("SL".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("SN".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
            <RegionMap<T>>::insert("TG".as_bytes().to_vec(), "AFW".as_bytes().to_vec());

            <RegionMap<T>>::insert("BW".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
            <RegionMap<T>>::insert("LS".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
            <RegionMap<T>>::insert("NA".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
            <RegionMap<T>>::insert("SZ".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
            <RegionMap<T>>::insert("ZA".as_bytes().to_vec(), "AFS".as_bytes().to_vec());

            <RegionMap<T>>::insert("AX".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("DK".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("EE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("FI".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("FO".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("GB".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("GG".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("IE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("IM".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("IS".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("JE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("LT".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("LV".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("NO".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("SE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
            <RegionMap<T>>::insert("SJ".as_bytes().to_vec(), "EUN".as_bytes().to_vec());

            <RegionMap<T>>::insert("BG".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("BY".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("CZ".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("HU".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("MD".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("PL".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("RO".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("RU".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("SK".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
            <RegionMap<T>>::insert("UA".as_bytes().to_vec(), "EUE".as_bytes().to_vec());

            <RegionMap<T>>::insert("AT".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("BE".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("CH".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("DE".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("FR".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("LI".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("LU".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("MC".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
            <RegionMap<T>>::insert("NL".as_bytes().to_vec(), "EUW".as_bytes().to_vec());

            <RegionMap<T>>::insert("AD".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("AL".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("BA".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("ES".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("GI".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("GR".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("HR".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("IT".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("ME".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("MK".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("MT".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("PT".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("RS".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("SI".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("SM".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("VA".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
            <RegionMap<T>>::insert("XK".as_bytes().to_vec(), "EUS".as_bytes().to_vec());

            <RegionMap<T>>::insert("AS".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("CK".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("NU".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("PF".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("PN".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("TK".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("TO".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("TV".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("WF".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
            <RegionMap<T>>::insert("WS".as_bytes().to_vec(), "OCP".as_bytes().to_vec());

            <RegionMap<T>>::insert("AU".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
            <RegionMap<T>>::insert("CC".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
            <RegionMap<T>>::insert("CX".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
            <RegionMap<T>>::insert("NF".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
            <RegionMap<T>>::insert("NZ".as_bytes().to_vec(), "OCA".as_bytes().to_vec());

            <RegionMap<T>>::insert("FJ".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
            <RegionMap<T>>::insert("NC".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
            <RegionMap<T>>::insert("PG".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
            <RegionMap<T>>::insert("SB".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
            <RegionMap<T>>::insert("VU".as_bytes().to_vec(), "OCM".as_bytes().to_vec());

            <RegionMap<T>>::insert("FM".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
            <RegionMap<T>>::insert("GU".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
            <RegionMap<T>>::insert("KI".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
            <RegionMap<T>>::insert("MH".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
            <RegionMap<T>>::insert("MP".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
            <RegionMap<T>>::insert("NR".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
            <RegionMap<T>>::insert("PW".as_bytes().to_vec(), "OCN".as_bytes().to_vec());

            <RegionMapInit<T>>::put(true);
        }
    }

    impl<T: Config> NodeInterface<T::AccountId, T::BlockNumber> for Pallet<T> {
        fn get_onboard_time(account_id: &T::AccountId) -> Option<T::BlockNumber> {
            Self::onboard_time(account_id)
        }

        fn im_ever_online(account_id: &T::AccountId) -> bool {
            Self::get_im_online(account_id) != None
        }

        fn get_eras_offline(account_id: &T::AccountId) -> u32 {
            let block = Self::get_im_online(account_id).unwrap_or(T::BlockNumber::default());
            let current_block = <frame_system::Pallet<T>>::block_number();
            let eras = (current_block - block) / T::BlocksPerEra::get();
            TryInto::<u32>::try_into(eras).ok().unwrap()
        }
    }
}
