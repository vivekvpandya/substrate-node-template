#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod traits;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::Dispatchable,
		pallet_prelude::*,
		traits::{schedule::Named as ScheduleNamed, LockIdentifier},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::boxed::Box;
	const GC_ID: LockIdentifier = *b"garbagec";

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + Sized {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type CleanupCall: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
		type PalletsOrigin: From<frame_system::RawOrigin<Self::AccountId>>;
		type Scheduler: ScheduleNamed<Self::BlockNumber, Self::CleanupCall, Self::PalletsOrigin>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// #[pallet::storage]
	// #[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CleanupCallScheduled { index: u64, priority: u64 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		SchedulerError,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn schedule_cleanup(
			origin: OriginFor<T>,
			index: u32,
			when: T::BlockNumber,
			call: Box<T::CleanupCall>,
		) -> DispatchResult {
			let _who = ensure_signed(origin);
			// NOTE: perhaps it makes sense to store call in storage once registered
			// and then schedule it from on_initialize(). Because maybe_periodic requires number
			// for how many time scheduled call should execute periodically and after that it
			// removes it from the scheduler.
			let _ = T::Scheduler::schedule_named(
				(GC_ID, index).encode(),
				frame_support::traits::schedule::DispatchTime::At(when),
				None,
				63,
				frame_system::RawOrigin::Root.into(),
				*call,
			)
			.map_err(|_| Error::<T>::SchedulerError)?;
			Self::deposit_event(Event::<T>::CleanupCallScheduled {
				index: index.into(),
				priority: 63,
			});
			Ok(())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: sp_std::marker::PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {}
	}
}
