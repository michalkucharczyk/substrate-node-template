#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Currency;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::Codec;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Currency;
	use frame_support::traits::ReservableCurrency;
    use frame_support::traits::ExistenceRequirement;
	use frame_support::BoundedVec;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	type BalanceOf<T> =
		<<T as Config>::Token as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//https://paritytech.github.io/substrate/master/frame_support/traits/tokens/currency/trait.Currency.html
		type Token: ReservableCurrency<Self::AccountId>;
		type ServiceProviderIdentity: Parameter + Into<u32> + MaxEncodedLen;
		type ServiceIdentity: Parameter + Into<u32> + MaxEncodedLen;
		type SubscriptionPeriod: Parameter
			+ Into<<Self as frame_system::Config>::BlockNumber>
			+ From<<Self as frame_system::Config>::BlockNumber>
			+ MaxEncodedLen;
		type SubscriptionFee: Parameter + MaxEncodedLen + Into<BalanceOf<Self>>;

		#[pallet::constant]
		type MaxServicesPerProvider: Get<u16>;
		#[pallet::constant]
		type MaxUserSubscriptions: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//todo
		SampleEvent,
	}

	#[pallet::error]
	pub enum Error<T> {
		ServiceProviderAlreadyRegistered,
		ServiceProviderNotRegistered,
		ServiceNotKnown,
		CannotRegisterService,
		ServiceAlreadyRegistered,
		UserAlreadySubscribed,
		CannotSubscribeUserMaxSubscriptions,
		UserNotSubscribed,
        InsufficientBalance, //todo: cleanup
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	//not needed - use System::block_number() (and most likely System::set_block_number() in tests).
	pub type CurrentBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	//keeps track of all service providers
	pub(super) type ServiceProviders<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ServiceProviderIdentity, (), OptionQuery>;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	//Service registered by service provider. Keeps basic service data.
	pub struct ServiceInfo<ServiceIdentity, SubscriptionPeriod, AccountId, SubscriptionFee> {
		pub id: ServiceIdentity,
		pub period: SubscriptionPeriod,
		pub account: AccountId,
		pub fee: SubscriptionFee,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	//Services subscribed by user
	pub struct SubscriptionInfo<ServiceProviderIdentity, ServiceIdentity> {
		pub service_provider: ServiceProviderIdentity,
		pub service: ServiceIdentity,
	}

	// type ServicesVec<T:Config> = BoundedVec< ServiceInfo<<T as Config>::ServiceIdentity,<T as Config>::SubscriptionPeriod,<T as frame_system::Config>::AccountId, <T as Config>::SubscriptionFee>, <T as Config>::MaxServicesPerProvider>;
	//
	// #[pallet::storage]
	// pub(super) type Services<T: Config> = StorageMap<_, Blake2_128Concat, T::ServiceProviderIdentity, ServicesVec<T>, OptionQuery>;

	#[pallet::storage]
	//Keeps track of all services for given service providers
	//(ServiceProviderIdentity, ServiceIdentity) --> { service_identity, period, fee, receiver_account }
	pub(super) type Services2<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::ServiceProviderIdentity,
		Blake2_128Concat,
		T::ServiceIdentity,
		ServiceInfo<
			<T as Config>::ServiceIdentity,
			<T as Config>::SubscriptionPeriod,
			<T as frame_system::Config>::AccountId,
			<T as Config>::SubscriptionFee,
		>,
		OptionQuery,
	>;

	// For renewing subscriptions the following is optimal:
	// (BlockNumber, AccountId) -> [Subscriptions]
	// Otherwise we would need to iterate over each user's accounts in each block.
	#[pallet::storage]
	pub(super) type Subscriptions<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::BlockNumber,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<
			SubscriptionInfo<
				<T as Config>::ServiceProviderIdentity,
				<T as Config>::ServiceIdentity,
			>,
			T::MaxUserSubscriptions,
		>,
		ValueQuery,
	>;

	// for canceling or checking if user is already subscribed this mapping allows to avoid iterating over all BlockNumbers in Subscriptions map
	// (can be thought of as kind of the on-chain cache)
	// This increases the state, but limits the size of PoV (as proof is smaller).
	// AccountId                -> [BlockNumber]
	#[pallet::storage]
	pub(super) type UserSubscriptions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::BlockNumber, T::MaxUserSubscriptions>,
		ValueQuery,
	>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Execute the scheduled calls
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut total_weight: Weight = Weight::default();
			CurrentBlockNumber::<T>::set(now);
            Self::renew_subscriptions(now);
			total_weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn register_service_provider(
			origin: OriginFor<T>,
			service_provider: T::ServiceProviderIdentity,
		) -> DispatchResult {
			ensure!(
				!ServiceProviders::<T>::contains_key(&service_provider),
				Error::<T>::ServiceProviderAlreadyRegistered
			);
			ServiceProviders::<T>::insert(&service_provider, ());

			// ensure!(!Services::<T>::contains_key(&service_provider), Error::<T>::ServiceProviderAlreadyRegistered);
			// Services::<T>::insert(&service_provider, ServicesVec::<T>::default());

			Ok(())
		}

		//todo: ServiceInfo in function call
		#[pallet::weight(1_000)]
		pub fn register_service(
			origin: OriginFor<T>,
			service_provider: T::ServiceProviderIdentity,
			service: T::ServiceIdentity,
			period: T::SubscriptionPeriod,
			receiver_account: T::AccountId,
			fee: T::SubscriptionFee,
		) -> DispatchResult {
			// ensure!(Services::<T>::contains_key(&service_provider), Error::<T>::ServiceProviderNotRegistered);
			// Services::<T>::insert(&service_provider, ServicesVec::<T>::default());

			Self::is_service_provider_registered(&service_provider)?;

			println!("b:{:?}", Services2::<T>::get(&service_provider, &service));

			ensure!(
				Services2::<T>::iter_prefix_values(&service_provider).count()
					< T::MaxServicesPerProvider::get() as usize,
				Error::<T>::CannotRegisterService
			);

			Services2::<T>::try_mutate::<_, _, _, Error<T>, _>(
				&service_provider,
				&service,
				|x| match x {
					Some(_) => Err(Error::<T>::ServiceAlreadyRegistered),
					None => {
						*x = Some(ServiceInfo {
							id: service.clone(),
							period: period,
							account: receiver_account,
							fee,
						});
						Ok(())
					},
				},
			)?;

			println!("a:{:?}", Services2::<T>::get(&service_provider, &service));
			println!("a:{:?}", Services2::<T>::iter().collect::<Vec<_>>());

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn subscribe(
			origin: OriginFor<T>,
			service_provider: T::ServiceProviderIdentity,
			service: T::ServiceIdentity,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = CurrentBlockNumber::<T>::get();

			//check if service is registered
			Self::is_service_registered(&service_provider, &service)?;
            let service_info = Services2::<T>::get(&service_provider, &service).expect("is_service_registered was check.qed");

			//check if already subscribed
			let user_renewal_blocks = UserSubscriptions::<T>::get(&who);

			println!("now: {} user_renewal_blocks: {:?}", now, user_renewal_blocks);
			println!("        subs:{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());

			let already_subscribed = user_renewal_blocks.iter().any(|b| {
				Subscriptions::<T>::get(&b, &who)
					.iter()
					.any(|sub| sub.service == service && sub.service_provider == service_provider)
			});

			ensure!(!already_subscribed, Error::<T>::UserAlreadySubscribed);

			//check the balance
            ensure!(T::Token::free_balance(&who) >= service_info.fee.clone().into(), Error::<T>::InsufficientBalance);


            let next_renewal = now + service_info.period.into();

			//push new subscription to user's subscriptions
			Subscriptions::<T>::try_mutate(&next_renewal, &who, |subs| -> DispatchResult {
				subs.try_push(SubscriptionInfo { service_provider, service })
					.map_err(|_| Error::<T>::CannotSubscribeUserMaxSubscriptions)?;
				Ok(())
			})?;

			//push 'now+service_info.period' to user's blocks
			UserSubscriptions::<T>::try_mutate(&who, |user_renewal_blocks| -> DispatchResult {
				if !user_renewal_blocks.iter().any(|b| *b == next_renewal) {
					user_renewal_blocks
						.try_push(next_renewal)
						.map_err(|_| Error::<T>::CannotSubscribeUserMaxSubscriptions)?;
				}
				Ok(())
			})?;

			//take the fee
            T::Token::transfer(&who, &service_info.account, service_info.fee.into(), ExistenceRequirement::AllowDeath)?;

			Ok(())
		}

		#[pallet::weight(1_000)]
		pub fn cancel(
			origin: OriginFor<T>,
			service_provider: T::ServiceProviderIdentity,
			service: T::ServiceIdentity,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//find subscription in pending BlockNumber bucket
			let user_renewal_blocks = UserSubscriptions::<T>::get(&who);

			println!("user_renewal_blocks: {:?}", user_renewal_blocks);
			println!("subs:{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());

			let bucket = user_renewal_blocks.iter().enumerate().find(|(i, b)| {
				let subs = Subscriptions::<T>::get(b, &who);
				subs.iter()
					.find(|sub| sub.service == service && sub.service_provider == service_provider)
					.is_some()
			});

			println!("bucket: {:?}", bucket);

			ensure!(bucket.is_some(), Error::<T>::UserNotSubscribed);

			let bucket = bucket.expect("Already ensured that is_some. qed");

			Subscriptions::<T>::try_mutate(bucket.1, &who, |subs| -> DispatchResult {
				subs.retain(|s| !(s.service_provider == service_provider && s.service == service));
				Ok(())
			})?;

			if let subs = Subscriptions::<T>::get(bucket.1, &who) {
				if subs.is_empty() {
					Subscriptions::<T>::remove(bucket.1, &who);
				}
			}

			UserSubscriptions::<T>::try_mutate(&who, |b| -> DispatchResult {
				b.retain(|i| i != bucket.1);
				Ok(())
			})?;

			println!("subs :{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());
			println!("block:{:?}", UserSubscriptions::<T>::iter().collect::<Vec<_>>());

			Ok(())
		}
	}
}

use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
impl<T: Config> Pallet<T> {
	fn is_service_provider_registered(
		service_provider: &T::ServiceProviderIdentity,
	) -> DispatchResult {
		ensure!(
			ServiceProviders::<T>::contains_key(&service_provider),
			Error::<T>::ServiceProviderNotRegistered
		);
		Ok(())
	}

	fn is_service_registered(
		service_provider: &T::ServiceProviderIdentity,
		service: &T::ServiceIdentity,
	) -> DispatchResult {
		Self::is_service_provider_registered(service_provider)?;
		ensure!(
			Services2::<T>::contains_key(&service_provider, &service),
			Error::<T>::ServiceNotKnown
		);
		Ok(())
	}

    fn renew_subscriptions(
        block_number: T::BlockNumber,
        ) -> DispatchResult {
        Ok(())
    }

}
