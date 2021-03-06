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
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::AtLeast32BitUnsigned,
		sp_runtime::traits::Hash, // support T::Hashing
		sp_runtime::SaturatedConversion,
		traits::{
			Currency, ExistenceRequirement,
			ExistenceRequirement::{AllowDeath, KeepAlive},
			Randomness,
		},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::ops::Add;
	use scale_info::prelude::string::String; // support String
	use scale_info::prelude::vec::Vec;
	use scale_info::TypeInfo; // support Vec

	use frame_support::traits::UnixTime; // support Timestamp

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
	use frame_system::offchain::{CreateSignedTransaction, SubmitTransaction};
	use pallet_bo_liquidity::BoLiquidityInterface;
	use pallet_symbol_price::SymbolPriceInterface;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the BoTrading pallet.
		type Currency: Currency<Self::AccountId>;

		/// Loose coupling with BoLiquidity pallet
		type BoLiquidity: BoLiquidityInterface<Self::AccountId>;

		/// Loose coupling with SymbolPrice pallet
		type SymbolPriceModule: SymbolPriceInterface;

		/// Use for create random data
		type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// Use get current time
		type TimeProvider: UnixTime;

		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/*
	Add Order info
	 */
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum TradeType {
		Call,
		Put,
	}

	/// User will trade on these pair
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum CurrencyPair {
		BtcUsdt,
		DotUsdc,
		BtcEth,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum OrderStatus {
		// Pending, // Is for frontend or other service only
		/// The order was created on the chain, and is being scheduled for checking result
		Created,
		/// The order expired_at was reach, it mean this is the time for checking order result, if it win or lose
		/// Normally Created will be changed to Win/Lose if all the operation was fast enough
		/// But in reality, we need to get the price from oracle, then check if it win or lose might take time, so we must use this Expired status
		Checking,
		/// Completed = Win / Lose
		Win,
		Lose,
	}

	pub type SymbolPrice = u128;

	/// Struct for holding Order information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Order<T: Config> {
		pub id: T::Hash,
		pub user_id: AccountOf<T>,
		pub currency_pair: CurrencyPair,
		pub trade_type: TradeType,
		// TODO: Ask: scale_info::TypeInfo do not support float,
		// So What is the best approach to working with float data (eg: trading volume) when we wanna store it on-chain? (use u64, use Balances pallet, etc)
		/// trading volume in unit of the stable coin
		///
		/// For example: User buy 0.01 XXX token => trading volume is 0.01
		/// Can I use pallet balance to store trading volume like the bellow code?
		/// impl pallet_bo_trading::Config for Runtime {
		/// 	type Event = Event;
		/// 	type Currency = Balances; // pallet balance
		/// }
		/// type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
		pub volume_in_unit: BalanceOf<T>,
		pub expired_at: u64,
		pub created_at: u64,
		pub liquidity_pool_id: AccountOf<T>,
		pub payout_rate: u32, // percent (1-100): the win rate of the LP at the open time
		pub open_price: SymbolPrice,
		pub close_price: Option<SymbolPrice>,
		pub status: OrderStatus,
	}

	/*
	End Order info
	 */

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	// #[pallet::storage]
	// #[pallet::getter(fn something)]
	// // Learn more about declaring storage items:
	// // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// pub type Something<T> = StorageValue<_, u32>;

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
	pub(super) type UserOrders<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<T::Hash>, ValueQuery>;

	/// Defines the block when next unsigned transaction will be accepted.
	///
	/// To prevent spam of unsigned (and unpayed!) transactions on the network,
	/// we only allow one transaction every `T::UnsignedInterval` blocks.
	/// This storage entry defines when new transaction is going to be accepted.
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at)]
	pub(super) type NextUnsignedAt<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// /// Event documentation should end with an array that provides descriptive names for event
		// /// parameters. [something, who]
		// SomethingStored(u32, T::AccountId),
		/// The order was created
		/// parameters. [sender, order_id]
		OrderCreated(T::AccountId, T::Hash),

		OrderClosed {
			account_id: T::AccountId,
			order_id: T::Hash,
			close_price: u128,
			status: OrderStatus,
			amount_payout: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// /// Error names should be descriptive.
		// NoneValue,
		// /// Errors should have helpful documentation associated with them.
		// StorageOverflow,
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
		/// Order Not Exist
		OrderNotExist,
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let valid_tx = |provide| {
				// Now let's check if the transaction has any chance to succeed.
				let next_unsigned_at = <NextUnsignedAt<T>>::get();
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				ValidTransaction::with_tag_prefix("bo_trading_crond")
					// set priority to 2^18
					.priority(1 << 18 + Self::blocknumber_to_u64(current_block_number).unwrap()) // please define `UNSIGNED_TXS_PRIORITY` before this line
					.and_provides([&provide])
					.longevity(5)
					.propagate(true)
					.build()
			};

			match call {
				Call::close_order {
					block_number: current_block_number,
					ref order_id,
					ref close_price,
				} => valid_tx(b"close_order".to_vec()),
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Offchain Worker entry point.
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("BoTrading Offchain workers: block_number: {:?}", block_number);
			let res = Self::scan_and_validate_expired_order_raw_unsigned(block_number);
			if let Err(e) = res {
				log::error!("Error: {}", e);
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// /// An example dispatchable that takes a singles value as a parameter, writes the value to
		// /// storage and emits an event. This function must be dispatched by a signed extrinsic.
		// #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
		// 	// Check that the extrinsic was signed and get the signer.
		// 	// This function will return an error if the extrinsic is not signed.
		// 	// https://docs.substrate.io/v3/runtime/origins
		// 	let who = ensure_signed(origin)?;
		//
		// 	// Update storage.
		// 	<Something<T>>::put(something);
		//
		// 	let lp = T::BoLiquidity::get_suitable_lp();
		//
		// 	// Emit an event.
		// 	Self::deposit_event(Event::SomethingStored(something, who));
		// 	// Return a successful DispatchResultWithPostInfo
		// 	Ok(())
		// }
		//
		// /// An example dispatchable that may throw a custom error.
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		//
		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => Err(Error::<T>::NoneValue)?,
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }

		/// Create an order
		///  - volume_in_unit: 2 decimal place, eg: 1000 mean 10.00
		///  - expired_at: unix timestamp
		///
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
			let CURRENCY_DECIMAL = 12;
			// TODO: Get this setting from LiquidityPool setting
			let MIN_TRADING_VOL = 1; // min trading vol is 1 token = (~$1)
			let MAX_TRADING_VOL = 1000; // max trading vol is 1000 token (~$1000)

			let PAYOUT_RATE = 95; // payout rate to user

			let min_vol_in_unit: BalanceOf<T> =
				Self::u64_to_balance(MIN_TRADING_VOL * 10_u64.pow(CURRENCY_DECIMAL))
					.ok_or(<Error<T>>::InvalidTradingVolume)?;
			let max_vol_in_unit: BalanceOf<T> =
				Self::u64_to_balance(MAX_TRADING_VOL * 10_u64.pow(CURRENCY_DECIMAL))
					.ok_or(<Error<T>>::InvalidTradingVolume)?;

			log::info!("min_vol is {:?} and max_vol is {:?}.", min_vol_in_unit, max_vol_in_unit);

			ensure!(min_vol_in_unit.le(&volume_in_unit), <Error<T>>::InvalidTradingVolume);
			ensure!(max_vol_in_unit.ge(&volume_in_unit), <Error<T>>::InvalidTradingVolume);

			let current_ts: u64 = T::TimeProvider::now().as_secs(); // TODO: Get current timestamp
			log::info!("Order is creating at {:?} and expired at {:?}.", current_ts, expired_at);

			ensure!(current_ts < expired_at, <Error<T>>::InvalidExpiredAt);

			// Check the buyer has enough free balance to place this order
			ensure!(
				T::Currency::free_balance(&sender) >= volume_in_unit,
				<Error<T>>::NotEnoughBalance
			);

			// Performs this operation first as it may fail
			let new_cnt: u64 =
				Self::order_count().checked_add(1).ok_or(<Error<T>>::OrderCountOverflow)?;

			// TODO: Ensure: Allow a specific currency only!

			// select a pool id for this order
			let volumn = Self::balance_to_u64(volume_in_unit).unwrap();
			let suitable_lp_id = T::BoLiquidity::get_suitable_lp(volumn);
			ensure!(suitable_lp_id.is_some(), <Error<T>>::NoLiquidityPool);

			let open_price =
				(T::SymbolPriceModule::get_price(String::from("BTC_USDT").into_bytes())).unwrap();

			// create orders
			let mut order = Order::<T> {
				id: T::Hashing::hash_of(&b"N/A"),
				user_id: sender.clone(),
				currency_pair,
				trade_type,
				volume_in_unit,
				expired_at,
				created_at: current_ts,
				liquidity_pool_id: suitable_lp_id.unwrap(), // unwrap is safe because of `ensure` check above
				payout_rate: PAYOUT_RATE,                   // fix 95% or must get this from liquidity pool info
				open_price,
				close_price: None,
				status: OrderStatus::Created,
			};

			// TODO: Ask: Is this too complex? How can we improve this?
			// TODO: switch to randomize id because hashing this way do not ensure uniqueness
			let order_id = T::Hashing::hash_of(&order);
			order.id = order_id;

			// ---- Save to db ------
			// Performs this operation first because as it may fail
			// <UserOrders<T>>::try_mutate(&sender, |vec| {
			// 	vec.push(order_id);
			// 	Ok(())
			// }).map_err(|_| <Error<T>>::CannotSaveUserOrders)?;
			<UserOrders<T>>::append(sender.clone(), order_id);
			<Orders<T>>::insert(order_id, order.clone());
			<OrderCount<T>>::put(new_cnt);

			// Deposit balance user
			T::Currency::transfer(
				&sender.clone(),
				&order.liquidity_pool_id,
				volume_in_unit,
				ExistenceRequirement::KeepAlive,
			)?;

			// Update LP balance
			T::BoLiquidity::update_lp_balance(order.liquidity_pool_id);

			log::info!("Order created: {:?}.", order_id);
			Self::deposit_event(Event::OrderCreated(sender, order_id));

			Ok(())
		}

		/// Validate, finish this order
		/// - Determine this is win or loose
		/// - So dome money transfer logic
		#[pallet::weight(1_000 + T::DbWeight::get().writes(1))]
		pub fn close_order(
			origin: OriginFor<T>,
			block_number: T::BlockNumber,
			order_id: T::Hash,
			close_price: SymbolPrice,
		) -> DispatchResult {
			let CURRENCY_DECIMAL = 12;
			// Get Order
			let order = Orders::<T>::get(&order_id);
			let mut status = OrderStatus::Lose;
			match order {
				Some(ref order) => {
					// Check result
					match order.trade_type {
						TradeType::Call => {
							if order.open_price < close_price {
								status = OrderStatus::Win;
							}
						}
						TradeType::Put => {
							if order.open_price > close_price {
								status = OrderStatus::Win;
							}
						}
					}
				}
				None => Err(Error::<T>::OrderNotExist)?,
			}

			// Set status
			let mut order = order.clone().unwrap();
			order.status = status.clone();
			order.close_price = Some(close_price);

			let mut volumn_payout: BalanceOf<T> = Self::u64_to_balance(0).unwrap();

			// Update info Orders
			Orders::<T>::try_mutate_exists(&order_id, |order| -> DispatchResult {
				let mut order = order.as_mut().ok_or(Error::<T>::OrderNotExist)?;
				order.status = status.clone();
				order.close_price = Some(close_price);
				Ok(())
			})?;

			match status {
				OrderStatus::Win => {
					volumn_payout = (order.volume_in_unit
						* Self::u64_to_balance(u64::from(100 + order.payout_rate)).unwrap())
						/ Self::u64_to_balance(100).unwrap();

					log::info!("volumn_payout: {:?}", volumn_payout);
					// Payout
					T::Currency::transfer(
						&order.liquidity_pool_id,
						&order.user_id,
						volumn_payout,
						ExistenceRequirement::KeepAlive,
					)?;

					// Update LP balance
					T::BoLiquidity::update_lp_balance(order.liquidity_pool_id);
				}
				_ => {
					log::info!("Lose: order_id, close_price: {:?}, {:?}", order_id, close_price);
				}
			}

			/*
			let sender = ensure_signed(origin)?;
			log::info!("close_order: order_id, close_price: {:?}, {:?}", order_id, close_price);
			Self::deposit_event(Event::OrderClosed {
				account_id: sender,
				order_id,
				close_price,
				status,
				amount_payout: volumn_payout,
			});
			*/

			log::info!("close_order: order_id, close_price: {:?}, {:?}", order_id, close_price);
			Self::deposit_event(Event::OrderClosed {
				account_id: order.user_id,
				order_id,
				close_price,
				status,
				amount_payout: volumn_payout,
			});

			<NextUnsignedAt<T>>::put(block_number + T::UnsignedInterval::get());

			// Update info Orders
			// Orders::<T>::try_mutate_exists(&order_id, |order| -> DispatchResult {
			// 	let mut order = order.as_mut().ok_or(Error::<T>::OrderNotExist)?;
			// 	Ok(())
			// })?;
			// Orders::<T>::try_mutate_exists(&order_id, |_| -> DispatchResult {
			// 	let _order = order;
			// 	Ok(())
			// })?;

			// TODO: Remove order in Orders -> push to OrderCompleted

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

		pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		// How to convert BlockNumber <=> u64 :
		pub fn blocknumber_to_u64(input: T::BlockNumber) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		/// This is crond entry point:
		///	- scan for expired tx from on-chain data
		/// - liquid it out by sending a transaction to on-chain
		///
		pub fn scan_and_validate_expired_order_raw_unsigned(
			block_number: T::BlockNumber,
		) -> Result<(), &'static str> {
			// Get current timestamp
			let current_ts = T::TimeProvider::now().as_secs();
			log::info!("scan and validate order: block_number: {:?}", block_number);

			// Scan all Orders
			for order in Orders::<T>::iter_values().filter(|s| s.status == OrderStatus::Created) {
				// Check order expired and call close_order
				if current_ts >= order.expired_at {
					let close_price =
						T::SymbolPriceModule::get_price(String::from("BTC_USDT").into_bytes())
							.unwrap();

					log::info!("Call close_order in block_number: {:?}", block_number);
					// Create a call close_order
					let call = Call::close_order { block_number, order_id: order.id, close_price };

					// submit the call to on-chain
					SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
						.map_err(|()| "Unable to submit unsigned transaction.")?;
				}
			}

			Ok(())
		}
	}
}
