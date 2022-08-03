#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

use frame_support::traits::Currency;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Currency;
    use frame_support::traits::ExistenceRequirement;
    use frame_support::traits::ReservableCurrency;
    use frame_support::BoundedVec;
    use frame_system::pallet_prelude::*;

    type BalanceOf<T> =
        <<T as Config>::Token as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Currency used to deduct the fee
        type Token: ReservableCurrency<Self::AccountId>;

        /// Service provider identity
        type ServiceProviderIdentity: Parameter + Into<u32> + MaxEncodedLen;

        /// Service identity
        type ServiceIdentity: Parameter + Into<u32> + MaxEncodedLen;

        /// Subscription period, counted in blocks
        type SubscriptionPeriod: Parameter
            + Into<<Self as frame_system::Config>::BlockNumber>
            + MaxEncodedLen;

        /// Subscription fee for the subscription period
        type SubscriptionFee: Parameter + MaxEncodedLen + Into<BalanceOf<Self>>;

        #[pallet::constant]
        /// Number of services allowed for service_provider
        type MaxServicesPerProvider: Get<u32>;

        #[pallet::constant]
        /// Number of subscriptions allowed for user
        type MaxUserSubscriptions: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        //todo: implement events
        ServiceProviderRegistered(T::ServiceProviderIdentity),
        ServiceRegistered(T::ServiceProviderIdentity, T::ServiceIdentity),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Service provider is alread registered
        ServiceProviderAlreadyRegistered,

        /// Service provider is not known
        ServiceProviderNotRegistered,

        /// Service is not known
        ServiceNotKnown,

        /// Serivce is already registered for service provider
        CannotRegisterService,

        /// Serivce is already registered for service provider
        ServiceAlreadyRegistered,

        /// User is already subscribed to service
        UserAlreadySubscribed,

        /// User already reached maximum number of subscriptions
        CannotSubscribeUserMaxSubscriptions,

        /// User is not subscribed to the given service
        UserNotSubscribed,

        /// Insufficient balance, subscription cannot be made
        InsufficientBalance, //todo: cleanup, use Balance::InsufficientBalance?
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    //todo: not needed - use System::block_number() (and most likely System::set_block_number() in tests).
    pub type CurrentBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::storage]
    /// Keeps track of all service providers
    pub(super) type ServiceProviders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ServiceProviderIdentity, (), OptionQuery>;

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    /// Service registered by service provider. Keeps basic service data.
    pub struct ServiceInfo<ServiceIdentity, SubscriptionPeriod, AccountId, SubscriptionFee> {
        pub id: ServiceIdentity,
        pub period: SubscriptionPeriod,
        pub account: AccountId,
        pub fee: SubscriptionFee,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    /// Service subscribed by user 
    pub struct SubscriptionInfo<ServiceProviderIdentity, ServiceIdentity> {
        pub service_provider: ServiceProviderIdentity,
        pub service: ServiceIdentity,
    }

    #[pallet::storage]
    /// Keeps track of all services for given service providers
    /// (ServiceProviderIdentity, ServiceIdentity) --> { service_identity, period, fee, receiver_account }
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

    /// For renewing subscriptions the following structure is optimal:
    /// (BlockNumber, AccountId) -> [Subscriptions]
    /// Otherwise we would need to iterate over each user's accounts in each block.
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

    /// For canceling or checking if user is already subscribed this mapping allows to avoid iterating over all BlockNumbers in Subscriptions map
    /// (can be thought of as kind of the on-chain cache)
    /// This increases the state size, but limits the size of PoV (as proof is smaller).
    /// AccountId -> [BlockNumber]
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
        /// Execute all the renewals for this particular block number
        fn on_initialize(now: T::BlockNumber) -> Weight {
            //todo: compute total_weight
            let total_weight: Weight = Weight::default();
            CurrentBlockNumber::<T>::set(now);
            Self::renew_subscriptions(now);
            total_weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        /// Registers new service provider.
        pub fn register_service_provider(
            _origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
        ) -> DispatchResult {
            //todo: take deposit
            ensure!(
                !ServiceProviders::<T>::contains_key(&service_provider),
                Error::<T>::ServiceProviderAlreadyRegistered
            );
            ServiceProviders::<T>::insert(&service_provider, ());

            Self::deposit_event(Event::<T>::ServiceProviderRegistered (service_provider));

            Ok(())
        }

        //todo: put struct ServiceInfo in function call
        #[pallet::weight(1_000)]
        /// Registers new service for given service provider.
        pub fn register_service(
            _origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
            service: T::ServiceIdentity,
            period: T::SubscriptionPeriod,
            receiver_account: T::AccountId,
            fee: T::SubscriptionFee,
        ) -> DispatchResult {
            //todo: take deposit
            Self::is_service_provider_registered(&service_provider)?;

            // println!("b:{:?}", Services2::<T>::get(&service_provider, &service));

            ensure!(
                Services2::<T>::iter_prefix_values(&service_provider).count()
                    < T::MaxServicesPerProvider::get() as usize,
                Error::<T>::CannotRegisterService
            );

            Services2::<T>::try_mutate::<_, _, _, Error<T>, _>(&service_provider, &service, |x| {
                match x {
                    Some(_) => Err(Error::<T>::ServiceAlreadyRegistered),
                    None => {
                        *x = Some(ServiceInfo {
                            id: service.clone(),
                            period,
                            account: receiver_account,
                            fee,
                        });
                        Ok(())
                    },
                }
            })?;

            // println!("a:{:?}", Services2::<T>::get(&service_provider, &service));
            // println!("a:{:?}", Services2::<T>::iter().collect::<Vec<_>>());

            Self::deposit_event(Event::<T>::ServiceRegistered (service_provider, service));

            Ok(())
        }

        #[pallet::weight(1_000)]
        /// Subscribes user for given service. The subscription will be automatically renewed given
        /// that sufficient funds will be on the account at the end of subscription period.
        pub fn subscribe(
            origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
            service: T::ServiceIdentity,
        ) -> DispatchResult {
            //todo: implement no-fee logic in case of successful subscriptions

            let who = ensure_signed(origin)?;
            let now = CurrentBlockNumber::<T>::get();

            //check if service is registered
            Self::is_service_registered(&service_provider, &service)?;
            let service_info = Services2::<T>::get(&service_provider, &service)
                .expect("is_service_registered was check.qed");

            //check if already subscribed
            let user_renewal_blocks = UserSubscriptions::<T>::get(&who);

            // println!("now: {} user_renewal_blocks: {:?}", now, user_renewal_blocks);
            // println!("        subs:{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());

            let already_subscribed = user_renewal_blocks.iter().any(|b| {
                Subscriptions::<T>::get(&b, &who)
                    .iter()
                    .any(|sub| sub.service == service && sub.service_provider == service_provider)
            });

            ensure!(!already_subscribed, Error::<T>::UserAlreadySubscribed);

            //check the balance
            ensure!(
                T::Token::free_balance(&who) >= service_info.fee.clone().into(),
                Error::<T>::InsufficientBalance
            );

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
            T::Token::transfer(
                &who,
                &service_info.account,
                service_info.fee.into(),
                ExistenceRequirement::AllowDeath,
            )?;

            Ok(())
        }

        #[pallet::weight(1_000)]
        /// Cancels user's subscription for given service. 
        pub fn cancel(
            origin: OriginFor<T>,
            service_provider: T::ServiceProviderIdentity,
            service: T::ServiceIdentity,
        ) -> DispatchResult {
            //todo: implement no-fee logic in case of successful cancellation
            let who = ensure_signed(origin)?;

            //find subscription in pending BlockNumber bucket
            let user_renewal_blocks = UserSubscriptions::<T>::get(&who);

            // println!("user_renewal_blocks: {:?}", user_renewal_blocks);
            // println!("subs:{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());

            let bucket = user_renewal_blocks.iter().find(|b| {
                Subscriptions::<T>::get(b, &who)
                    .iter()
                    .find(|sub| sub.service == service && sub.service_provider == service_provider)
                    .is_some()
            });

            ensure!(bucket.is_some(), Error::<T>::UserNotSubscribed);

            let bucket = bucket.expect("Already ensured that is_some. qed");

            Subscriptions::<T>::try_mutate(bucket, &who, |subs| -> DispatchResult {
                subs.retain(|s| !(s.service_provider == service_provider && s.service == service));
                Ok(())
            })?;

            if Subscriptions::<T>::get(bucket, &who).is_empty() {
                Subscriptions::<T>::remove(bucket, &who);
            }

            UserSubscriptions::<T>::try_mutate(&who, |b| -> DispatchResult {
                b.retain(|i| i != bucket);
                Ok(())
            })?;

            // println!("subs :{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());
            // println!("block:{:?}", UserSubscriptions::<T>::iter().collect::<Vec<_>>());

            Ok(())
        }
    }
}

//question: why do I need to duplicate it?
use frame_support::ensure;
use frame_support::pallet_prelude::DispatchResult;
use frame_support::traits::ExistenceRequirement;
use frame_support::traits::Get;

//question: is this good practice?
impl<T: Config> Pallet<T> {
    #[cfg(test)]

    /// Assuming that user is subscribed to service_provider's service, returns the block number
    /// for which subscription will be renewed
    fn get_renewal_block_for_user(
        who: T::AccountId,
        service_provider: T::ServiceProviderIdentity,
        service: T::ServiceIdentity,
    ) -> Option<T::BlockNumber> {
        UserSubscriptions::<T>::get(&who)
            .iter()
            .find(|b| {
                Subscriptions::<T>::get(b, &who)
                    .iter()
                    .find(|sub| sub.service == service && sub.service_provider == service_provider)
                    .is_some()
            })
            .map(|b| *b)
    }

    /// Checks if given service_provider is registered
    fn is_service_provider_registered(
        service_provider: &T::ServiceProviderIdentity,
    ) -> DispatchResult {
        ensure!(
            ServiceProviders::<T>::contains_key(&service_provider),
            Error::<T>::ServiceProviderNotRegistered
        );
        Ok(())
    }

    /// Checks if given service is registered
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

    /// Executes the renewal process for the given block. If user has sufficient funds then the fee
    /// is transferred to service's account, and user's subscription info is moved to the bucket at
    /// the end of subscription period.
    fn renew_subscriptions(block_number: T::BlockNumber) {
        //todo: we shall also checks the buckets which are less then current block_number (this should not happen theoretically).
        //todo: error handling shall be reviewed.
        let users = Subscriptions::<T>::iter_prefix(block_number)
            .map(|(who, _)| who)
            .collect::<Vec<_>>();

        users.iter().for_each(|who| {
            //remove all subscriptions for user for given block_number
            let subscription_infos = Subscriptions::<T>::take(block_number, who);

            //remove block-number from user's blocks
            let _ = UserSubscriptions::<T>::try_mutate(&who, |b| -> Result<(), ()> {
                b.retain(|i| *i != block_number);
                Ok(())
            });

            assert!(subscription_infos.len() < <T as Config>::MaxUserSubscriptions::get() as usize);
            assert!(
                UserSubscriptions::<T>::get(&who).len()
                    < <T as Config>::MaxUserSubscriptions::get() as usize
            );

            //take subscription fee, and add subscription Subscriptions and new block_number to user's block_numbers
            subscription_infos.iter().for_each(|info| {
                let service_info = Services2::<T>::get(&info.service_provider, &info.service)
                    .expect("service cannot be removed yet. qed");

                if T::Token::free_balance(&who) >= service_info.fee.clone().into() {
                    //schedule renewal for the end of next period:
                    let next_renewal = block_number + service_info.period.into();
                    let _ = Subscriptions::<T>::try_mutate(
                        &next_renewal,
                        &who,
                        |subs| -> Result<(), ()> {
                            subs.try_push(info.clone()).expect("there is space for one item. qed");
                            Ok(())
                        },
                    );

                    //push 'now+service_info.period' to user's blocks
                    let _ = UserSubscriptions::<T>::try_mutate(
                        &who,
                        |user_renewal_blocks| -> Result<(), ()> {
                            if !user_renewal_blocks.iter().any(|b| *b == next_renewal) {
                                user_renewal_blocks
                                    .try_push(next_renewal)
                                    .expect("there is space for one item. qed");
                            }
                            Ok(())
                        },
                    );

                    //take the fee, todo: is it safe to discard result here?
                    let _ = T::Token::transfer(
                        &who,
                        &service_info.account,
                        service_info.fee.into(),
                        ExistenceRequirement::AllowDeath,
                    );
                }
            });
        });

        // println!("subs :{:?}", Subscriptions::<T>::iter().collect::<Vec<_>>());
        // println!("block:{:?}", UserSubscriptions::<T>::iter().collect::<Vec<_>>());
    }
}
