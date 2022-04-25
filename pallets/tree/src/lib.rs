#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{log, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_garbage_collecter::traits::{Cleanable, CleanableAction};
	use sp_runtime::offchain::{storage::StorageValueRef, StorageKind};

	const OFFCAIN_INDEX_KEY: &[u8] = b"tree::storage::height";

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn height)]
	pub type Height<T> = StorageValue<_, u32, ValueQuery>;
	#[pallet::storage]
	#[pallet::getter(fn last_watering_time)]
	pub type LastWateringTime<T: Config> =
		StorageValue<_, <T as frame_system::Config>::BlockNumber, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		HeightIncreased(u32),
		HeightDecreased(u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		OcwError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn cleanup(origin: OriginFor<T>) -> DispatchResult {
			let _ = ensure_root(origin)?;
			if <Self as Cleanable>::cleanable() {
				<Self as CleanableAction>::cleanable_action();
			}
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn water(origin: OriginFor<T>) -> DispatchResult {
			let _ = ensure_root(origin)?;
			let h = Height::<T>::get();
			let h = h.saturating_add(5);
			Height::<T>::set(h);
			Self::deposit_event(Event::<T>::HeightIncreased(h));
			Ok(())
		}
	}

	impl<T: Config> Cleanable for Pallet<T> {
		fn cleanable() -> bool {
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let last_watering_block = LastWateringTime::<T>::get();
			current_block_number > (last_watering_block + 5_u32.into())
		}
	}

	impl<T: Config> CleanableAction for Pallet<T> {
		fn cleanable_action() {
			let h = Height::<T>::get();
			// assuming old height value needed to be sent to some http server
			// store it in offchain index.
			sp_io::offchain_index::set(&OFFCAIN_INDEX_KEY, &h.encode());
			let h = h.saturating_sub(5);
			Height::<T>::set(h);
			Self::deposit_event(Event::<T>::HeightDecreased(h));
		}
	}

	impl<T: Config> Pallet<T> {
		fn send_data<R: serde::Serialize>(request_body: R) -> Result<Vec<u8>, Error<T>> {
			// Load http URI from offchain storage
			// this should have been configured on start up by passing e.g. `--http-uri`
			// e.g. `--http-uri=http://localhost:8545`
			// if above method does not work then perhpas use offchain_localStorageSet RPC endpoint
			// or hardcode value here
			let http_uri = if let Some(value) =
				sp_io::offchain::local_storage_get(StorageKind::PERSISTENT, b"HTTP_URI")
			{
				value
			} else {
				return Err(Error::<T>::OcwError)
			};
			let http_uri = core::str::from_utf8(&http_uri).map_err(|_| Error::<T>::OcwError)?;
			const HEADER_CONTENT_TYPE: &str = "application/json";
			let body =
				serde_json::to_string::<R>(&request_body).map_err(|_| Error::<T>::OcwError)?;
			let raw_body = body.as_bytes();
			let request = sp_runtime::offchain::http::Request::post(http_uri, vec![raw_body]);
			let timeout = sp_io::offchain::timestamp()
				.add(sp_runtime::offchain::Duration::from_millis(3000_u64));
			let pending_req = request
				.add_header("Content-Type", HEADER_CONTENT_TYPE)
				.add_header("Content-Length", &raw_body.len().to_string())
				.deadline(timeout)
				.send()
				.map_err(|_| Error::<T>::OcwError)?;

			// returning value is Result of Result so need to unwrap twice with ? operator.
			let response = pending_req
				.try_wait(timeout)
				.map_err(|_| Error::<T>::OcwError)?
				.map_err(|_| Error::<T>::OcwError)?;
			if response.code != 200 {
				return Err(Error::<T>::OcwError)
			}

			Ok(response.body().collect::<Vec<u8>>())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub last_watering_block: T::BlockNumber,
		pub height: u32,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig::<T> { last_watering_block: 0_u32.into(), height: 5_u32 }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			LastWateringTime::<T>::set(self.last_watering_block);
			Height::<T>::set(self.height);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_n: T::BlockNumber) {
			// check if offchain indexing value set, then send it to http server
			let oci_mem = StorageValueRef::persistent(&OFFCAIN_INDEX_KEY);
			if let Ok(Some(h)) = oci_mem.get::<u32>() {
				if let Ok(response) = Self::send_data(h) {
					log::info!("SUCCESS: send tree heigt to server");
					log::info!("Response: {:?}", response);
					// clear storage
					sp_io::offchain_index::clear(&OFFCAIN_INDEX_KEY);
				} else {
					log::error!("FAILURE: send tree heigt to server");
				}
			}
		}
	}
}
