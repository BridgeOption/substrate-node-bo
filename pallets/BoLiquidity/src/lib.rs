#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;
//
#[cfg(test)]
mod tests;
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
			tokens::ExistenceRequirement,
		},
		PalletId,
	};

	use sp_runtime::traits::AccountIdConversion;

	use scale_info::TypeInfo;
	// use scale_info::prelude::string::String; // support String
	use scale_info::prelude::vec::Vec;	// support Vec

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the BoTrading pallet.
		type Currency: Currency<Self::AccountId>;

		/// Use for create random data
		type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
        type PalletId: Get<PalletId>;
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
		pub id: T::AccountId,
		pub name: Vec<u8>,
		pub amount: BalanceOf<T>,
		pub payout_rate: u8,
		pub admin: T::AccountId,
	}

	#[pallet::storage]
	#[pallet::getter(fn lp_count)]
	pub type LpCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pools)]
	/// Stores list of created lp
	pub(super) type LiquidityPools<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, LiquidityPool<T>>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pools_owned)]
	/// Keeps track of what accounts own what LP.
	pub(super) type LiquidityPoolsOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pools_index)]
	pub(crate) type LiquidityPoolsIndex<T: Config> = StorageMap<_, Twox64Concat, u32, T::AccountId, OptionQuery>;

	// TODO: Multi-sig for this pool


	#[pallet::storage]
	#[pallet::getter(fn lp_items_rank)]
	/// Stores list of created LP Rank
	pub(super) type LpItemsRank<T: Config> = StorageMap<_, Twox64Concat, LpRank, Vec<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn lp_items_rank_index)]
	/// Stores list of created LP Rank
	pub(super) type LpItemsRankIndex<T: Config> = StorageMap<_, Twox64Concat, LpRank, Vec<u32>>;

	#[pallet::storage]
	#[pallet::getter(fn lp_random_index)]
	pub(super) type LpRandomIndex<T: Config> = StorageValue<_, u32, ValueQuery>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The order was created
		/// parameters. [sender, lp_id]
		LPCreated(T::AccountId, T::AccountId),
		LPDeposit(T::AccountId, T::AccountId),
		LPGetRandom(T::AccountId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Handles arithemtic overflow when incrementing the LP counter.
		LPCntOverflow,
		/// Handles checking amount when create LP
		InvalidAmount,
		/// Handles checking whether the LP doest not exists.
		NoLiquidityPool,
		/// Not enough balance to create lp
		NotEnoughBalance,
		/// Handles checking payout rate when create LP
		InvalidPayoutRate,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_lp(origin: OriginFor<T>, name: Vec<u8>, payout_rate: u8, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let min_amount: BalanceOf<T> = Self::u64_to_balance(10000).ok_or(<Error<T>>::InvalidAmount)?;
			ensure!(amount.ge(&min_amount), <Error<T>>::InvalidAmount);

			let new_cnt = Self::lp_count().checked_add(1)
				.ok_or(<Error<T>>::LPCntOverflow)?;
			let current_lp_idx = new_cnt - 1;

			// let pallet_account_id = Self::account_id();
			let lp_id = Self::sub_account_id(current_lp_idx);

			let mut liquidity_pool = LiquidityPool::<T> {
				id: lp_id.clone(),
				name: name,
				amount: amount,
				payout_rate: payout_rate,
				admin: sender.clone(),
			};

			ensure!(payout_rate > 0, <Error<T>>::InvalidPayoutRate);
			ensure!(payout_rate < 100, <Error<T>>::InvalidPayoutRate);

			// Check if the lp does not already exist in our storage map
			ensure!(Self::liquidity_pools(&lp_id) == None, <Error<T>>::NoLiquidityPool);

			// Check the buyer has enough free balance to create this lp
			ensure!(T::Currency::free_balance(&sender) >= amount, <Error<T>>::NotEnoughBalance);

			// amount need larger than ExistentialDeposit const define in Runtime
			T::Currency::transfer(&sender, &lp_id, amount, ExistenceRequirement::KeepAlive)?;

			<LiquidityPools<T>>::insert(lp_id.clone(), liquidity_pool);
			<LpCount<T>>::put(new_cnt);
			<LiquidityPoolsOwned<T>>::append(sender.clone(), lp_id.clone());
			<LiquidityPoolsIndex<T>>::insert(&current_lp_idx, lp_id.clone());

			let lp_rank = Self::get_lprank(amount);
			<LpItemsRank<T>>::append(&lp_rank, lp_id.clone());
			<LpItemsRankIndex<T>>::append(lp_rank, current_lp_idx);

			Self::deposit_event(Event::LPCreated(sender, lp_id.clone()));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn deposit_lp(origin: OriginFor<T>, lp_id: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let min_amount: BalanceOf<T> = Self::u64_to_balance(10000).ok_or(<Error<T>>::InvalidAmount)?;
			ensure!(amount.ge(&min_amount), <Error<T>>::InvalidAmount);

			// Check the buyer has enough free balance to create this lp
			ensure!(T::Currency::free_balance(&sender) >= amount, <Error<T>>::NotEnoughBalance);

			LiquidityPools::<T>::try_mutate_exists(&lp_id, |liquidity_pool| -> DispatchResult {
				let mut lp = liquidity_pool.as_mut().ok_or(Error::<T>::NoLiquidityPool)?;
				lp.amount = lp.amount + amount;

				// amount need larger than ExistentialDeposit const define in Runtime
				T::Currency::transfer(&sender, &lp_id, amount, ExistenceRequirement::KeepAlive)?;

				Ok(())
			})?;

			Self::deposit_event(Event::LPDeposit(sender, lp_id.clone()));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_lp(origin: OriginFor<T>, lp_id: T::AccountId, name: Vec<u8>, payout_rate: u8) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			LiquidityPools::<T>::try_mutate_exists(&lp_id, |liquidity_pool| -> DispatchResult {
				let mut lp = liquidity_pool.as_mut().ok_or(Error::<T>::NoLiquidityPool)?;
				let owner = lp.admin.clone();
				ensure!(owner == sender, "You are not the owner lp");
				ensure!(payout_rate > 0, <Error<T>>::InvalidPayoutRate);
				ensure!(payout_rate < 100, <Error<T>>::InvalidPayoutRate);

				lp.name = name;
				lp.payout_rate = payout_rate;

				Ok(())
			})?;

			Self::deposit_event(Event::LPDeposit(sender, lp_id.clone()));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn get_lp(origin: OriginFor<T>, volumn: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let lp_id = Self::pick_a_suitable_lp(volumn);
			ensure!(lp_id.is_some(), <Error<T>>::NoLiquidityPool);

			// log::info!("Random LP: {:?}.", lp_id.clone().unwrap());

			Self::deposit_event(Event::LPGetRandom(sender, lp_id.clone().unwrap()));

			Ok(())
		}
		
	}


	/// Internal helpers fn
	impl<T: Config> Pallet<T> {

		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}

		/// The account ID of a sub account
		pub fn sub_account_id(id: u32) -> T::AccountId {
			T::PalletId::get().into_sub_account(("bt", id))
		}

		pub fn u64_to_balance(input: u64) -> Option<BalanceOf<T>> {
			input.try_into().ok()
		}

		pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
			TryInto::<u64>::try_into(input).ok()
		}

		/// Get LP rank by amount
		fn get_lprank (amount: BalanceOf<T>) -> LpRank {
			let amount_u64 = Self::balance_to_u64(amount).unwrap();

			if amount_u64 < 10000 {
				return LpRank::Inactive;
			} else if amount_u64 >= 10000 && amount_u64 < 500000 {
				return LpRank::Tiny;
			} else if amount_u64 >= 500000 {
				return LpRank::Earth;
			} else {
				return LpRank::Inactive;
			}
		}

		/// Randomly choose a LP from among the total number of lps.
		/// Returns `None` if there are no lp.
		fn choose_lp(total: u32) -> Option<u32> {
			if total == 0 {
				return None
			}

			let mut pool_index = LpRandomIndex::<T>::get();
			log::info!("current pool_index: {:?}.", pool_index);
			let total = LpCount::<T>::get();

			pool_index = (pool_index + 1) % total;
			LpRandomIndex::<T>::put(pool_index);
			log::info!("new pool_index: {:?}.", pool_index);

			Some(pool_index)

			// let mut random_number = Self::generate_random_number(0);
	
			// // Best effort attempt to remove bias from modulus operator.
			// for i in 1..<LpCount<T>>::get() {
			// 	if random_number < u32::MAX - u32::MAX % total {
			// 		break
			// 	}
	
			// 	random_number = Self::generate_random_number(i);
			// }
	
			// Some(random_number % total)
		}

		/// Generate a random number from a given seed.
		fn generate_random_number(seed: u32) -> u32 {
			let (random_seed, _) = T::MyRandomness::random(&("bo_liquidity", seed).encode());

			let random_number = <u32>::decode(&mut random_seed.as_ref())
				.expect("secure hashes should always be bigger than u32; qed");
			random_number
		}

		/// Get a unique hash to use as lp id
		fn get_next_lp_id() -> Option<T::AccountId> {
			// Use Round robin get random LP
			match Self::choose_lp(<LpCount<T>>::get()) {
				None => None,
				Some(lp) => <LiquidityPoolsIndex<T>>::get(lp),
			}
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
		fn pick_a_suitable_lp(volumn:u64) -> Option<T::AccountId> {
			// TODO: Round robin or implement a suitable approach to get suitable LP
			// And improve the picking speed
			Self::get_next_lp_id()
		}
	}

	///
	/// Expose for loosely coupling
	/// for using in other pallet
	///
	pub trait BoLiquidityInterface<THashType> {
		fn get_suitable_lp(volumn:u64) -> Option<THashType>;
	}

	// impl<T: Config> BoLiquidityInterface for Module<T> {
	impl<T: Config> BoLiquidityInterface<T::AccountId> for Pallet<T> {
		// use Pallet<T> instead of Module<T> to support calling in other impl of Pallet?
		fn get_suitable_lp(volumn:u64) -> Option<T::AccountId> {
			Self::pick_a_suitable_lp(volumn)
		}
	}
	// End loosely coupling
}
