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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_garbage_collecter::traits::{Cleanable, CleanableAction};

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
	pub type LastWateringTime<T: Config> = StorageValue<_, <T as frame_system::Config>::BlockNumber, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		HeightIncreased(u32),
		HeightDecreased(u32),
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn cleanup(
			origin: OriginFor<T>
		) -> DispatchResult {
			// ensure root origin
			ensure_root(origin)?;
			if <Self as Cleanable>::cleanable() {
				<Self as CleanableAction>::cleanable_action();
			}
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn water(
				origin: OriginFor<T>,
			) -> DispatchResult {
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
			if current_block_number > (last_watering_block + 5_u32.into()) {
				true
			} else {
				false
			}
		}
	}

	impl<T: Config> CleanableAction for Pallet<T> {
		fn cleanable_action() {
			let h = Height::<T>::get();
			let h = h.saturating_sub(5);
			Height::<T>::set(h);
			Self::deposit_event(Event::<T>::HeightDecreased(h));
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T :Config> {
		pub last_watering_block : T::BlockNumber,
		pub height: u32,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig::<T> {
				last_watering_block: 0_u32.into(),
				height: 5_u32,
			}
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
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let weight : Weight = 0_u32.into();
			if <Self as Cleanable>::cleanable() {
				//use sp_runtime::offchain::http;
				// send some data here
			}
			weight
		}
	}

}
