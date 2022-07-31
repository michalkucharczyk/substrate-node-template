#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

use frame_support::{RuntimeDebug};
use frame_support::traits::{Currency};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use frame_support::traits::ReservableCurrency;
	use frame_support::traits::Currency;
    use frame_support::dispatch::Codec;

    type BalanceOf<T> = <<T as Config>::Token as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        //https://paritytech.github.io/substrate/master/frame_support/traits/tokens/currency/trait.Currency.html
        type Token: ReservableCurrency<Self::AccountId>;
        type ServiceProviderIdentity: Parameter + Into<u32> + MaxEncodedLen;
        type ServiceIdentity: Parameter + Into<u32> + MaxEncodedLen;
        type SubscriptionPeriod: Parameter + From<<Self as frame_system::Config>::BlockNumber> + MaxEncodedLen;
        type SubscriptionFee: Parameter + MaxEncodedLen /* + Get<BalanceOf<Self>> */;

        #[pallet::constant]
        type MaxServicesPerProvider: Get<u32>;
    }


    // Pallets use events to inform users when important changes are made.
    // Event documentation should end with an array that provides descriptive names for parameters.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SampleEvent,
    }

    #[pallet::error]
    pub enum Error<T> {
        ServiceProviderAlreadyRegistered,
        ServiceProviderNotRegistered,
        CannotRegisterService,
        ServiceAlreadyRegistered,
    }

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type ServiceProviders<T: Config> = StorageMap<_, Blake2_128Concat, T::ServiceProviderIdentity, (), OptionQuery>;

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct ServiceInfo<ServiceIdentity, SubscriptionPeriod, AccountId, SubscriptionFee> {
        pub id: ServiceIdentity,
        pub sp: SubscriptionPeriod,
        pub account: AccountId,
        pub fee: SubscriptionFee,
    }

    // type ServicesVec<T:Config> = BoundedVec< ServiceInfo<<T as Config>::ServiceIdentity,<T as Config>::SubscriptionPeriod,<T as frame_system::Config>::AccountId, <T as Config>::SubscriptionFee>, <T as Config>::MaxServicesPerProvider>;
    //
    // #[pallet::storage]
    // pub(super) type Services<T: Config> = StorageMap<_, Blake2_128Concat, T::ServiceProviderIdentity, ServicesVec<T>, OptionQuery>;

    #[pallet::storage]
    pub(super) type Services2<T: Config> = StorageMap<_, Blake2_128Concat, (T::ServiceProviderIdentity, T::ServiceIdentity), 
        ServiceInfo<<T as Config>::ServiceIdentity,<T as Config>::SubscriptionPeriod,<T as frame_system::Config>::AccountId, <T as Config>::SubscriptionFee>, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Execute the scheduled calls
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut total_weight: Weight = Weight::default();
			total_weight
		}
	}

    // Dispatchable functions allow users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn register_service_provider(
            origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
        ) -> DispatchResult {

            ensure!(!ServiceProviders::<T>::contains_key(&service_provider), Error::<T>::ServiceProviderAlreadyRegistered);
            ServiceProviders::<T>::insert(&service_provider, ());

            // ensure!(!Services::<T>::contains_key(&service_provider), Error::<T>::ServiceProviderAlreadyRegistered);
            // Services::<T>::insert(&service_provider, ServicesVec::<T>::default());

            Ok(())
        }

        #[pallet::weight(1_000)]
        pub fn register_service(
            origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
            service: T::ServiceIdentity,
            period:  T::SubscriptionPeriod,
            receiver_account: T::AccountId,
            fee: T::SubscriptionFee
        ) -> DispatchResult {
            // ensure!(Services::<T>::contains_key(&service_provider), Error::<T>::ServiceProviderNotRegistered);
            // Services::<T>::insert(&service_provider, ServicesVec::<T>::default());

            ensure!(ServiceProviders::<T>::contains_key(&service_provider), Error::<T>::ServiceProviderNotRegistered);

            println!("b:{:?}", Services2::<T>::get((&service_provider,&service)));

            Services2::<T>::try_mutate::<_,_,Error<T>,_>( (&service_provider, &service.clone()), |x| {
                match x {
                    Some(_) => {
                        Err(Error::<T>::ServiceAlreadyRegistered)
                    },
                    None => {
                        *x = Some(ServiceInfo { id:service.clone() ,sp:period, account:receiver_account, fee: fee });
                        Ok(())
                    }
                }
            })?;

            println!("a:{:?}", Services2::<T>::get((&service_provider,&service)));
            println!("a:{:?}", Services2::<T>::iter().collect::<Vec<_>>());

            Ok(())
        }

        #[pallet::weight(1_000)]
        pub fn subscribe(
            origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
            service: T::ServiceIdentity
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::weight(1_000)]
        pub fn cancel(
            origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
            service: T::ServiceIdentity
        ) -> DispatchResult {
            Ok(())
        }
    }
}
