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
	// use frame_support::{
	// 	dispatch::DispatchResult,
	// 	pallet_prelude::*,
	// };
	use frame_system::pallet_prelude::*;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::Hash, // support T::Hashing
		traits::{
			// Randomness,
			Currency,
		},
	};
	use scale_info::TypeInfo;
	use scale_info::prelude::string::String; // support String
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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);



	/*
	Add Order info
	 */
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum TradeType { Call, Put }

	/// User will trade on these pair
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum CurrencyPair {
		BtcUsdt,
		DotUsdc,
		BtcEth,
	}


	/// Struct for holding Order information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Order<T: Config> {
		pub id: T::Hash,
		pub user_id: AccountOf<T>,
		pub currency_pair: CurrencyPair,
		pub trade_type: TradeType,
		// TODO: Ask: scale_info::TypeInfo do not support float,
		// So What is the best approach to save a float (eg: trading volume) inside db? u64? BalanceOf?
		/// trading volume in unit of the stable coin
		pub volume_in_unit: BalanceOf<T>,
		pub expired_at: u64,
		pub created_at: u64,
		pub liquidity_pool_id: T::Hash,
	}


	/*
	End Order info
	 */





	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;



	#[pallet::storage]
	#[pallet::getter(fn order_count)]
	pub type OrderCount<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	/// Stores list of created orders
	pub(super) type Orders<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Order<T>>;

	#[pallet::storage]
	#[pallet::getter(fn user_orders)]
	/// Keeps track of what accounts own what Order
	/// Ask: OptionQuery vs ValueQuery? What are there use cases?
	/// Answer: https://stackoverflow.com/a/69114934/4984888
	pub(super) type UserOrders<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<T::Hash>, ValueQuery>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),


		/// The order was created
		/// parameters. [sender, order_id]
		OrderCreated(T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,


		/// ExpiredAt must be a specific point in the future, and if timeframe is 5 minute
		InvalidExpiredAt,
		/// trading vol must be min / max
		InvalidTradingVolume,
		/// Not enough balance to place the order
		NotEnoughBalance,
		/// No suitable liquidity pool available for this order
		NoLiquidityPool,
		/// The queried order is not belong to current user
		OrderNotBelongToUser,
		/// overflow
		OrderCountOverflow,
		/// error during inserting order_id into user orders's vector
		CannotSaveUserOrders,
	}


	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}


		/// Create an order
		/// TODO: do benchmark to get this weight
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn place_order(
			origin: OriginFor<T>,
			currency_pair: CurrencyPair,
			trade_type: TradeType,
			volume_in_unit: BalanceOf<T>,
			expired_at: u64,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let sender = ensure_signed(origin)?;




			// ----- validation ------
			// TODO: Ask: how can I get decimal of currency, ie: 1 coin = 100...000 units
			let CURRENCY_DECIMAL = 18;
			// TODO: Get this setting from LiquidityPool setting
			let MIN_TRADING_VOL = 1; // min trading vol is 1 token = (~$1)
			let MAX_TRADING_VOL = 1000; // max trading vol is 1000 token (~$1000)

			let min_vol_in_unit: BalanceOf<T> = Self::u64_to_balance(MIN_TRADING_VOL * CURRENCY_DECIMAL).ok_or(<Error<T>>::InvalidTradingVolume)?;
			let max_vol_in_unit: BalanceOf<T> = Self::u64_to_balance(MAX_TRADING_VOL * CURRENCY_DECIMAL).ok_or(<Error<T>>::InvalidTradingVolume)?;

			ensure!(min_vol_in_unit.le(&volume_in_unit), <Error<T>>::InvalidTradingVolume);
			ensure!(max_vol_in_unit.ge(&volume_in_unit), <Error<T>>::InvalidTradingVolume);

			let current_ts: u64 = 0; // TODO: Get current timestamp
			ensure!(current_ts < expired_at, <Error<T>>::InvalidExpiredAt);

			// Check the buyer has enough free balance to place this order
			ensure!(T::Currency::free_balance(&sender) >= volume_in_unit, <Error<T>>::NotEnoughBalance);




			// Performs this operation first as it may fail
			let new_cnt: u64 = Self::order_count().checked_add(1).ok_or(<Error<T>>::OrderCountOverflow)?;


			// TODO: Ensure: Allow a specific currency only!

			let na_str = String::from("N/A").as_bytes().to_vec();

			// get pool
			let suitable_lp_id = T::Hashing::hash_of(&na_str);


			// create orders
			let mut order = Order::<T> {
				id: T::Hashing::hash_of(&na_str),
				user_id: sender.clone(),
				currency_pair,
				trade_type,
				volume_in_unit,
				expired_at,
				created_at: current_ts,
				liquidity_pool_id: suitable_lp_id,
			};

			// TODO: Ask: Is this too complex? How can we improve this?
			let order_id = T::Hashing::hash_of(&order);
			order.id = order_id;


			// ---- Save to db ------
			// Performs this operation first because as it may fail
			// <UserOrders<T>>::try_mutate(&sender, |vec| {
			// 	vec.push(order_id);
			// 	Ok(())
			// }).map_err(|_| <Error<T>>::CannotSaveUserOrders)?;
			<UserOrders<T>>::append(sender.clone(), order_id);
			<Orders<T>>::insert(order_id, order);
			<OrderCount<T>>::put(new_cnt);



			log::info!("Order created: {:?}.", order_id);
			Self::deposit_event(Event::OrderCreated(sender, order_id));

			Ok(())
		}
	}


	// pub const RAW_AMOUNT_SCALE: f64 = 100 as f64;

	/// Internal helpers fn
	impl<T: Config> Pallet<T> {
		// pub fn raw_amount_2_amount(raw_amount: u32) -> f64 {
		// 	(raw_amount / RAW_AMOUNT_SCALE) as f64
		// }
		//
		// pub fn amount_2_raw_amount(amount: f64) -> u32 {
		// 	amount * RAW_AMOUNT_SCALE
		// }


		pub fn hash_str<S: Encode>(s: &S) -> T::Hash {
			T::Hashing::hash_of(s)
		}

		// How to convert u64 <=> Balance : https://stackoverflow.com/a/56081118/4984888
		pub fn u64_to_balance(input: u64) -> Option<BalanceOf<T>> {
			input.try_into().ok()
		}
	}
}
