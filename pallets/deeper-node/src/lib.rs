#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::traits::{Currency, ReservableCurrency, Vec, Get};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type IpV4 = Vec<u8>;
pub type CountryRegion = Vec<u8>;
pub type Duration = u8;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    type MinLockAmt: Get<u32>;
    type MaxDurationDays: Get<u8>;
    type DayToBlocknum: Get<u32>;
    type MaxIpLength: Get<usize>;
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

// struct to store the registered Device Informatin
#[derive(Decode, Encode, Default)]
pub struct Node<AccountId, BlockNumber> {
    pub account_id: AccountId,
    ipv4: IpV4, // IP will not be exposed in future version
    country: CountryRegion,
    expire: BlockNumber,
}

// error messages
decl_error! {
    pub enum Error for Module<T: Trait> {
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
}

// events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        //Balance = BalanceOf<T>,
    {
        // register node: AccountId, ipv4, country
        RegisterNode(AccountId, IpV4, CountryRegion),
        UnregisterNode(AccountId),

        // add account into a country's server list
        ServerCountryAdded(AccountId, CountryRegion, BlockNumber, u64),
        // remove account from a country's server list
        ServerCountryRemoved(AccountId, CountryRegion),

        // add account into a region's server list
        ServerRegionAdded(AccountId, CountryRegion, BlockNumber, u64),
        // remove account from a region's server list
        ServerRegionRemoved(AccountId, CountryRegion),
    }
);

// storage for this module
decl_storage! {
    trait Store for Module<T: Trait> as Device {
        RegionMapInit get(fn get_map_init): bool = false;
        RegionMap get(fn get_region_code): map hasher(blake2_128_concat) CountryRegion => CountryRegion;
        DeviceInfo get(fn get_device_info): map hasher(blake2_128_concat) T::AccountId => Node<T::AccountId, T::BlockNumber>;
        ServersByCountry get(fn get_servers_by_country): map hasher(blake2_128_concat) CountryRegion => Vec<T::AccountId>;
        ServersByRegion get(fn get_servers_by_region): map hasher(blake2_128_concat) CountryRegion => Vec<T::AccountId>;
    }
    add_extra_genesis {
        build(|_| Module::<T>::setup_region_map())
    }
}

// public interface for this runtime module
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;
        // initialize the default event for this module
        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn register_device(origin, ip: IpV4, country: CountryRegion) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(RegionMapInit::get() == true, Error::<T>::InvalidRegionMap);
            ensure!(RegionMap::contains_key(&country), Error::<T>::InvalidCode);
            ensure!(ip.len() <= T::MaxIpLength::get(), Error::<T>::InvalidIP);

            if !<DeviceInfo<T>>::contains_key(&sender) {
                let node = Node {
                    account_id: sender.clone(),
                    ipv4: ip.clone(),
                    country: country.clone(),
                    expire: <frame_system::Module<T>>::block_number(),
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
                    node.expire = <frame_system::Module<T>>::block_number();
                });
            }
            Self::deposit_event(RawEvent::RegisterNode(sender, ip, country));
            Ok(())
        }

        #[weight = 10_000]
        pub fn unregister_device(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<DeviceInfo<T>>::contains_key(&sender), Error::<T>::DeviceNotRegister);
            let _ = Self::try_remove_server(&sender);
            <DeviceInfo<T>>::remove(&sender);
            T::Currency::unreserve(&sender,BalanceOf::<T>::from(T::MinLockAmt::get()));
            Self::deposit_event(RawEvent::UnregisterNode(sender));
            Ok(())
        }

        #[weight = 10_000]
        pub fn register_server(origin, duration: Duration) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<DeviceInfo<T>>::contains_key(&sender),
                    Error::<T>::DeviceNotRegister);
            ensure!(duration <= T::MaxDurationDays::get(), Error::<T>::DurationOverflow);
            let block_num = (duration as u32) * T::DayToBlocknum::get();
            let _ = Self::try_add_server(&sender, T::BlockNumber::from(block_num));
            Ok(())
        }

        #[weight = 10_000]
        pub fn update_server(origin, duration: Duration) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<DeviceInfo<T>>::contains_key(&sender),
                    Error::<T>::DeviceNotRegister);
            ensure!(duration <= T::MaxDurationDays::get(), Error::<T>::DurationOverflow);
            let block_num = (duration as u32) * T::DayToBlocknum::get();
            <DeviceInfo<T>>::mutate(&sender, |node| {
                node.expire = <frame_system::Module<T>>::block_number() + T::BlockNumber::from(block_num);
            });
            Ok(())
        }

        #[weight = 10_000]
        pub fn unregister_server(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(<DeviceInfo<T>>::contains_key(&sender),
                    Error::<T>::DeviceNotRegister);
            let _ = Self::try_remove_server(&sender);
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // try to remove an account from country and region server lists if exists
    fn try_remove_server(sender: &T::AccountId) -> DispatchResult {
        let mut node = <DeviceInfo<T>>::get(&sender);
        let first_region = RegionMap::get(&node.country);
        let sec_region = RegionMap::get(&first_region);

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
        node.expire = <frame_system::Module<T>>::block_number();
        <DeviceInfo<T>>::insert(&sender, node);

        Ok(())
    }

    // try to add an account to a country's server list; no double add
    fn try_add_server(sender: &T::AccountId, duration: T::BlockNumber) -> DispatchResult {
        let mut node = <DeviceInfo<T>>::get(&sender);
        let first_region = RegionMap::get(&node.country);
        let sec_region = RegionMap::get(&first_region);

        // country registration
        let mut country_server_list = <ServersByCountry<T>>::get(&node.country);
        if Self::country_list_insert(&mut country_server_list, &sender, &node.country, &duration) == false {
            Err(Error::<T>::DoubleCountryRegistration)?
        }

        // level 3 region registration
        let mut level3_server_list = <ServersByRegion<T>>::get(&first_region);
        if Self::region_list_insert(&mut level3_server_list, &sender, &first_region, &duration) == false {
            let _ = Self::country_list_remove(&mut country_server_list, &sender, &node.country);
            Err(Error::<T>::DoubleLevel3Registration)?
        }

        // level 2 region registration
        let mut level2_server_list = <ServersByRegion<T>>::get(&sec_region);
        if Self::region_list_insert(&mut level2_server_list, &sender, &sec_region, &duration) == false {
            let _ = Self::country_list_remove(&mut country_server_list, &sender, &node.country);
            let _ = Self::region_list_remove(&mut level3_server_list, &sender, &first_region);
            Err(Error::<T>::DoubleLevel2Registration)?
        }

        // ensure consistency
        node.expire = <frame_system::Module<T>>::block_number() + duration;
        <DeviceInfo<T>>::insert(&sender, node);

        Ok(())
    }


    fn country_list_insert(servers: &mut Vec<T::AccountId>, account: &T::AccountId, country: &CountryRegion, duration: &T::BlockNumber) -> bool {
        match servers.binary_search(&account) {
            Ok(_) => false,
            Err(index) => {
                servers.insert(index, account.clone());
                <ServersByCountry<T>>::insert(&country, servers);
                Self::deposit_event(RawEvent::ServerCountryAdded(
                    account.clone(),
                    country.clone(),
                    duration.clone(),
                    index as u64,
                ));
                true
            }
        }

    }

    fn country_list_remove(servers: &mut Vec<T::AccountId>, account: &T::AccountId, country: &CountryRegion) -> bool {
        match servers.binary_search(&account) {
            Ok(index) => {
                servers.remove(index);
                <ServersByCountry<T>>::insert(&country, servers);
                Self::deposit_event(RawEvent::ServerCountryRemoved(
                    account.clone(),
                    country.clone(),
                ));
                true
            },
            Err(_) => false,
        }
    }

    fn region_list_insert(servers: &mut Vec<T::AccountId>, account: &T::AccountId, region: &CountryRegion, duration: &T::BlockNumber) -> bool {
        match servers.binary_search(&account) {
            Ok(_) => false,
            Err(index) => {
                servers.insert(index, account.clone());
                <ServersByRegion<T>>::insert(&region, servers);
                Self::deposit_event(RawEvent::ServerRegionAdded(
                    account.clone(),
                    region.clone(),
                    duration.clone(),
                    index as u64,
                ));
                true
            }
        }

    }

    fn region_list_remove(servers: &mut Vec<T::AccountId>, account: &T::AccountId, region: &CountryRegion) -> bool {
        match servers.binary_search(&account) {
            Ok(index) => {
                servers.remove(index);
                <ServersByRegion<T>>::insert(&region, servers);
                Self::deposit_event(RawEvent::ServerRegionRemoved(
                    account.clone(),
                    region.clone(),
                ));
                true
            },
            Err(_) => false,
        }
    }

    fn setup_region_map() {
        /* level 1 */
        /*
        RegionMap::insert("AMER".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
        RegionMap::insert("ASIA".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
        RegionMap::insert("AFRI".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
        RegionMap::insert("EURO".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
        RegionMap::insert("OCEA".as_bytes().to_vec(), "ROOT".as_bytes().to_vec());
        */

        /* level 2 */
        RegionMap::insert("AMN".as_bytes().to_vec(), "AMER".as_bytes().to_vec());
        RegionMap::insert("AMC".as_bytes().to_vec(), "AMER".as_bytes().to_vec());
        RegionMap::insert("AMM".as_bytes().to_vec(), "AMER".as_bytes().to_vec());
        RegionMap::insert("AMS".as_bytes().to_vec(), "AMER".as_bytes().to_vec());

        RegionMap::insert("ASC".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
        RegionMap::insert("ASE".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
        RegionMap::insert("ASW".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
        RegionMap::insert("ASS".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());
        RegionMap::insert("ASD".as_bytes().to_vec(), "ASIA".as_bytes().to_vec());

        RegionMap::insert("AFN".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
        RegionMap::insert("AFM".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
        RegionMap::insert("AFE".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
        RegionMap::insert("AFW".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());
        RegionMap::insert("AFS".as_bytes().to_vec(), "AFRI".as_bytes().to_vec());

        RegionMap::insert("EUN".as_bytes().to_vec(), "EURO".as_bytes().to_vec());
        RegionMap::insert("EUE".as_bytes().to_vec(), "EURO".as_bytes().to_vec());
        RegionMap::insert("EUW".as_bytes().to_vec(), "EURO".as_bytes().to_vec());
        RegionMap::insert("EUS".as_bytes().to_vec(), "EURO".as_bytes().to_vec());

        RegionMap::insert("OCP".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());
        RegionMap::insert("OCA".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());
        RegionMap::insert("OCM".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());
        RegionMap::insert("OCN".as_bytes().to_vec(), "OCEA".as_bytes().to_vec());

        /* level 3 */
        RegionMap::insert("BM".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
        RegionMap::insert("CA".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
        RegionMap::insert("GL".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
        RegionMap::insert("PM".as_bytes().to_vec(), "AMN".as_bytes().to_vec());
        RegionMap::insert("US".as_bytes().to_vec(), "AMN".as_bytes().to_vec());

        RegionMap::insert("AG".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("AI".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("AW".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("BB".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("BL".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("BQ".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("BS".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("CU".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("CW".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("DM".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("DO".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("GD".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("GP".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("HT".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("JM".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("KN".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("KY".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("LC".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("MF".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("MQ".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("MS".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("PR".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("SX".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("TC".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("TT".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("VC".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("VG".as_bytes().to_vec(), "AMC".as_bytes().to_vec());
        RegionMap::insert("VI".as_bytes().to_vec(), "AMC".as_bytes().to_vec());

        RegionMap::insert("BZ".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("CR".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("GT".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("HN".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("MX".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("NI".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("PA".as_bytes().to_vec(), "AMM".as_bytes().to_vec());
        RegionMap::insert("SV".as_bytes().to_vec(), "AMM".as_bytes().to_vec());

        RegionMap::insert("AR".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("BO".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("BR".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("CL".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("CO".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("EC".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("FK".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("GF".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("GS".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("GY".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("PE".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("PY".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("SR".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("UY".as_bytes().to_vec(), "AMS".as_bytes().to_vec());
        RegionMap::insert("VE".as_bytes().to_vec(), "AMS".as_bytes().to_vec());

        RegionMap::insert("KG".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
        RegionMap::insert("KZ".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
        RegionMap::insert("TJ".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
        RegionMap::insert("TM".as_bytes().to_vec(), "ASC".as_bytes().to_vec());
        RegionMap::insert("UZ".as_bytes().to_vec(), "ASC".as_bytes().to_vec());

        RegionMap::insert("CN".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("HK".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("JP".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("KP".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("KR".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("MN".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("MO".as_bytes().to_vec(), "ASE".as_bytes().to_vec());
        RegionMap::insert("TW".as_bytes().to_vec(), "ASE".as_bytes().to_vec());

        RegionMap::insert("AE".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("AM".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("AZ".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("BH".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("CY".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("GE".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("IL".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("IQ".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("JO".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("KW".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("LB".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("OM".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("PS".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("QA".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("SA".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("SY".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("TR".as_bytes().to_vec(), "ASW".as_bytes().to_vec());
        RegionMap::insert("YE".as_bytes().to_vec(), "ASW".as_bytes().to_vec());

        RegionMap::insert("AF".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("BD".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("BT".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("IN".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("IR".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("LK".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("MV".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("NP".as_bytes().to_vec(), "ASS".as_bytes().to_vec());
        RegionMap::insert("PK".as_bytes().to_vec(), "ASS".as_bytes().to_vec());

        RegionMap::insert("BN".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("ID".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("KH".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("LA".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("MM".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("MY".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("PH".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("SG".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("TH".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("TL".as_bytes().to_vec(), "ASD".as_bytes().to_vec());
        RegionMap::insert("VN".as_bytes().to_vec(), "ASD".as_bytes().to_vec());

        RegionMap::insert("DZ".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
        RegionMap::insert("EG".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
        RegionMap::insert("LY".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
        RegionMap::insert("MA".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
        RegionMap::insert("SD".as_bytes().to_vec(), "AFN".as_bytes().to_vec());
        RegionMap::insert("TN".as_bytes().to_vec(), "AFN".as_bytes().to_vec());

        RegionMap::insert("AO".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("CD".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("CF".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("CG".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("CM".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("GA".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("GQ".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("ST".as_bytes().to_vec(), "AFM".as_bytes().to_vec());
        RegionMap::insert("TD".as_bytes().to_vec(), "AFM".as_bytes().to_vec());

        RegionMap::insert("BI".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("DJ".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("ER".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("ET".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("IO".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("KE".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("KM".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("MG".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("MU".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("MW".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("MZ".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("RE".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("RW".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("SC".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("SO".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("SS".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("TF".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("TZ".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("UG".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("YT".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("ZM".as_bytes().to_vec(), "AFE".as_bytes().to_vec());
        RegionMap::insert("ZW".as_bytes().to_vec(), "AFE".as_bytes().to_vec());

        RegionMap::insert("BF".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("BJ".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("CI".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("CV".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("GH".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("GM".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("GN".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("GW".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("LR".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("ML".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("MR".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("NE".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("NG".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("SH".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("SL".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("SN".as_bytes().to_vec(), "AFW".as_bytes().to_vec());
        RegionMap::insert("TG".as_bytes().to_vec(), "AFW".as_bytes().to_vec());

        RegionMap::insert("BW".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
        RegionMap::insert("LS".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
        RegionMap::insert("NA".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
        RegionMap::insert("SZ".as_bytes().to_vec(), "AFS".as_bytes().to_vec());
        RegionMap::insert("ZA".as_bytes().to_vec(), "AFS".as_bytes().to_vec());

        RegionMap::insert("AX".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("DK".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("EE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("FI".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("FO".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("GB".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("GG".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("IE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("IM".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("IS".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("JE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("LT".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("LV".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("NO".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("SE".as_bytes().to_vec(), "EUN".as_bytes().to_vec());
        RegionMap::insert("SJ".as_bytes().to_vec(), "EUN".as_bytes().to_vec());

        RegionMap::insert("BG".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("BY".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("CZ".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("HU".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("MD".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("PL".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("RO".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("RU".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("SK".as_bytes().to_vec(), "EUE".as_bytes().to_vec());
        RegionMap::insert("UA".as_bytes().to_vec(), "EUE".as_bytes().to_vec());

        RegionMap::insert("AT".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("BE".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("CH".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("DE".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("FR".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("LI".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("LU".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("MC".as_bytes().to_vec(), "EUW".as_bytes().to_vec());
        RegionMap::insert("NL".as_bytes().to_vec(), "EUW".as_bytes().to_vec());

        RegionMap::insert("AD".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("AL".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("BA".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("ES".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("GI".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("GR".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("HR".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("IT".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("ME".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("MK".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("MT".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("PT".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("RS".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("SI".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("SM".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("VA".as_bytes().to_vec(), "EUS".as_bytes().to_vec());
        RegionMap::insert("XK".as_bytes().to_vec(), "EUS".as_bytes().to_vec());

        RegionMap::insert("AS".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("CK".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("NU".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("PF".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("PN".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("TK".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("TO".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("TV".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("WF".as_bytes().to_vec(), "OCP".as_bytes().to_vec());
        RegionMap::insert("WS".as_bytes().to_vec(), "OCP".as_bytes().to_vec());

        RegionMap::insert("AU".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
        RegionMap::insert("CC".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
        RegionMap::insert("CX".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
        RegionMap::insert("NF".as_bytes().to_vec(), "OCA".as_bytes().to_vec());
        RegionMap::insert("NZ".as_bytes().to_vec(), "OCA".as_bytes().to_vec());

        RegionMap::insert("FJ".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
        RegionMap::insert("NC".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
        RegionMap::insert("PG".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
        RegionMap::insert("SB".as_bytes().to_vec(), "OCM".as_bytes().to_vec());
        RegionMap::insert("VU".as_bytes().to_vec(), "OCM".as_bytes().to_vec());

        RegionMap::insert("FM".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
        RegionMap::insert("GU".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
        RegionMap::insert("KI".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
        RegionMap::insert("MH".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
        RegionMap::insert("MP".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
        RegionMap::insert("NR".as_bytes().to_vec(), "OCN".as_bytes().to_vec());
        RegionMap::insert("PW".as_bytes().to_vec(), "OCN".as_bytes().to_vec());

        RegionMapInit::put(true);
    }

    pub fn registered_devices() -> Vec<Node<T::AccountId, T::BlockNumber>> {
        DeviceInfo::<T>::iter_values().collect::<Vec<_>>()
    }
}
