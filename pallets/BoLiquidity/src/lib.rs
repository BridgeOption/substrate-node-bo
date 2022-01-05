#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_system::pallet_prelude::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::Hash, // support T::Hashing
		traits::{
			Randomness,
			Currency,
		},
	};
	use scale_info::TypeInfo;
	// use scale_info::prelude::string::String; // support String
	use scale_info::prelude::vec::Vec;	// support Vec

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};





	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the BoTrading pallet.
		type Currency: Currency<Self::AccountId>;

		/// Use for create random data
		type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}



	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);



	///
	/// - Each LP will have size ranking:
	/// 	- LP_Inactive: < $100k
	/// 	- LP_Tiny: >= $100k
	/// 	- LP_Earth: >= $500k
	/// 	- LP_Moon: >= $1M
	/// 	- LP_Mars: >= $5M
	/// 	- LP_Jupiter: >= $10M
	/// 	- LP_Saturn: >= $20M
	/// 	- LP_Sun: >= $50M
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum LpRank {
		Inactive,
		Tiny,
		Earth,
		Moon,
		Mars,
		Jupiter,
		Saturn,
		Sun,
	}


	/// Struct for holding Liquidity information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct LiquidityPool<T: Config> {
		pub id: T::Hash,
	}



	#[pallet::storage]
	#[pallet::getter(fn lp_count)]
	pub type LpCount<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pools)]
	/// Stores list of created orders
	pub(super) type LiquidityPools<T: Config> = StorageMap<_, Twox64Concat, T::Hash, LiquidityPool<T>>;

	// TODO: Multi-sig for this pool


	#[pallet::storage]
	#[pallet::getter(fn lp_items_rank)]
	/// Stores list of created orders
	pub(super) type LpItemsRank<T: Config> = StorageMap<_, Twox64Concat, LpRank, Vec<T::Hash>>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The order was created
		/// parameters. [sender, lp_id]
		LPCreated(T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
	}



	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_lp(origin: OriginFor<T>, _something: u32) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			log::info!("Hello world: {:?}.", "oke");

			// Self::deposit_event(Event::LPCreated(something, who));

			Ok(())
		}
	}



	/// Internal helpers fn
	impl<T: Config> Pallet<T> {
		/// Get a unique hash to use as lp id
		fn get_next_lp_id() -> T::Hash {
			// TODO: Use Randomness
			return T::Hashing::hash_of(b"TODO");
		}

		/*
		Flow:
		- We manage the LP id with LP rank:
		LpItemsRank = {
			[LpRank]: [lp_id1, lp_id2, ...]
			[LpRank]: [lp_id100, lp_id999, ...]
		}
		So picking a LP from LpItemsRank will have O(1) time-complexity
		 */
		fn pick_a_suitable_lp() -> Option<T::Hash> {
			// TODO: Round robin or implement a suitable approach to get suitable LP
			// And improve the picking speed
			Some(T::Hashing::hash_of(b"TODO"))
		}
	}

	///
	/// Expose for loosely coupling
	/// for using in other pallet
	///
	pub trait BoLiquidityInterface<THashType> {
		fn get_suitable_lp() -> Option<THashType>;
	}

	// impl<T: Config> BoLiquidityInterface for Module<T> {
	impl<T: Config> BoLiquidityInterface<T::Hash> for Pallet<T> {
		// use Pallet<T> instead of Module<T> to support calling in other impl of Pallet?
		fn get_suitable_lp() -> Option<T::Hash> {
			Self::pick_a_suitable_lp()
		}
	}
	// End loosely coupling
}
