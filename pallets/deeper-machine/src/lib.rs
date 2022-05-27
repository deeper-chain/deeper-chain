#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_core::H160;
use pallet_evm::{AddressMapping, EnsureAddressOrigin};
use sp_runtime::AccountId32;
use frame_support::traits::IsType;
use frame_system::RawOrigin;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Eth Address => AccountId
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, H160, T::AccountId, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub account_pairs: std::collections::BTreeMap<H160, T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				account_pairs: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (eth_addr, account_id) in &self.account_pairs {
				<Accounts<T>>::insert(eth_addr, account_id);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	}

	#[pallet::error]
	pub enum Error<T> {
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
	}


}

pub trait DeeperMachineInterface<AccountId> {
	fn get_eth_address(account_id: &AccountId) -> Option<H160>;
}

impl<T: Config> DeeperMachineInterface<T::AccountId> for Pallet<T> {
	fn get_eth_address(account_id: &T::AccountId) -> Option<H160> {
		// TODO: read from storage
		None
	}
}

pub struct PairedAddressMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AddressMapping<T::AccountId> for PairedAddressMapping<T>
where
	T::AccountId: IsType<AccountId32>,
{
	fn into_account_id(address: H160) -> T::AccountId {
		if Accounts::<T>::contains_key(&address) {
			if let Some(acc) = Accounts::<T>::get(address) {
				return acc;
			}
		}

		let mut data: [u8; 32] = [0u8; 32];
		data[0..4].copy_from_slice(b"evm:");
		data[4..24].copy_from_slice(&address[..]);
		AccountId32::from(data).into()
	}
}

pub struct EnsureAddressPaired<T>(sp_std::marker::PhantomData<T>);

impl<OuterOrigin, T: Config> EnsureAddressOrigin<OuterOrigin> for EnsureAddressPaired<T>
where
	OuterOrigin: Into<Result<RawOrigin<T::AccountId>, OuterOrigin>> + From<RawOrigin<T::AccountId>>,
{
	type Success = T::AccountId;

	fn try_address_origin(
		address: &H160,
		origin: OuterOrigin,
	) -> Result<T::AccountId, OuterOrigin> {
		origin.into().and_then(|o| match o {
			RawOrigin::Signed(who) => {
				if let Some(acc) = Accounts::<T>::get(address) {
					if acc == who {
						return Ok(who);
					}
					return Err(OuterOrigin::from(RawOrigin::Signed(who)));
				}
				return Err(OuterOrigin::from(RawOrigin::Signed(who)));
			}
			r => Err(OuterOrigin::from(r)),
		})
	}
}