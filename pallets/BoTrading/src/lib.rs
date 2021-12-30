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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::{
		// sp_runtime::traits::Hash,
		traits::{
			Currency,
		},
	};
	use scale_info::TypeInfo;

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
	// type AmountOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		// So What is the best approach to save a float (eg: trading volume) inside db?
		/// trading volume in unit of the stable coin
		pub volume_in_unit: u32,
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
	#[pallet::getter(fn orderCount)]
	pub type OrderCount<T> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	/// Stores list of created orders
	pub(super) type Orders<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Order<T>>;

	#[pallet::storage]
	#[pallet::getter(fn userOrders)]
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
	}
}
