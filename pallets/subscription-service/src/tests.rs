use crate as pallet_service_subscription;
use crate::{mock::*, Error};
use frame_support::traits::OnInitialize;
use frame_support::{assert_noop, assert_ok, bounded_vec, BoundedVec};
use frame_system as system;
use sp_runtime::traits::{BlakeTwo256, ConstU32, IdentityLookup};
use frame_support::traits::tokens::currency::Currency;

type UserSubscriptionsVec = BoundedVec<
	<Test as system::Config>::AccountId,
	<Test as pallet_service_subscription::Config>::MaxUserSubscriptions,
>;

#[test]
fn cannot_register_service_provider_twice() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_noop!(
			ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42),
			Error::<Test>::ServiceProviderAlreadyRegistered
		);
	});
}

#[test]
fn register_single_service() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_ok!(
			ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 0, 10, 1, 99),
			()
		);
	});
}

#[test]
fn cannot_register_single_service_twice() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_ok!(
			ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 0, 10, 1, 99),
			()
		);
		assert_noop!(
			ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 0, 10, 1, 99),
			Error::<Test>::ServiceAlreadyRegistered
		);
	});
}

#[test]
fn register_two_services() {
	new_test_ext().execute_with(|| {
		let service_provider = 42;
		let service = 1;
		let fee = 99;
		let period = 10;
		let receiver_account = 1;

		assert_ok!(
			ServiceSubscriptionModule::register_service_provider(
				Origin::signed(1),
				service_provider
			),
			()
		);
		assert_ok!(
			ServiceSubscriptionModule::register_service(
				Origin::signed(1),
				service_provider,
				service + 0,
				period,
				receiver_account,
				fee
			),
			()
		);
		assert_ok!(
			ServiceSubscriptionModule::register_service(
				Origin::signed(1),
				service_provider,
				service + 1,
				period,
				receiver_account,
				fee
			),
			()
		);
	});
}

#[test]
fn fail_to_register_three_services() {
	new_test_ext().execute_with(|| {
		let service_provider = 42;
		let service = 1;
		let fee = 99;
		let period = 10;
		let receiver_account = 1;

		assert_ok!(
			ServiceSubscriptionModule::register_service_provider(
				Origin::signed(1),
				service_provider
			),
			()
		);
		assert_ok!(
			ServiceSubscriptionModule::register_service(
				Origin::signed(1),
				service_provider,
				service + 0,
				period,
				receiver_account,
				fee
			),
			()
		);
		assert_ok!(
			ServiceSubscriptionModule::register_service(
				Origin::signed(1),
				service_provider,
				service + 1,
				period,
				receiver_account,
				fee
			),
			()
		);

		//max 2 services per service provider, this shall fail:
		assert_noop!(
			ServiceSubscriptionModule::register_service(
				Origin::signed(1),
				service_provider,
				service + 2,
				period,
				receiver_account,
				fee
			),
			Error::<Test>::CannotRegisterService
		);
	});
}

#[test]
fn user_can_subscribe() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;
        let now = 10;

        Balances::make_free_balance_be(&2,2*99);

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(now);

		ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider);
		ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee);
		ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service+1, period, receiver_account, fee);

        assert_ok!(
            ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service),
            ()
        );
        assert_ok!(
            ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service+1),
            ()
        );

        let vb : UserSubscriptionsVec = bounded_vec![now+period];
        assert_eq!(pallet_service_subscription::pallet::UserSubscriptions::<Test>::get(2), vb);
	});
}

#[test]
fn user_exceeds_subscriptions_count() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);

        Balances::make_free_balance_be(&2,4*99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider+1));
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider+2));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service+1, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+1, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+1, service+1, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+2, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+2, service+1, period, receiver_account, fee));

		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service));
		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service+1));
		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider+1, service));
		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider+1, service+1));

		assert_noop!(
		    ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider+2, service+1),
            Error::<Test>::CannotSubscribeUserMaxSubscriptions
        );

	});
}

#[test]
fn user_already_subscribed_same_block() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);

        Balances::make_free_balance_be(&2,99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));

		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service));

		assert_noop!(
		    ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service),
            Error::<Test>::UserAlreadySubscribed
        );
	});
}

#[test]
fn user_already_subscribed_next_block() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);

        Balances::make_free_balance_be(&2,99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));

		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service));

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(11);
		assert_noop!(
		    ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service),
            Error::<Test>::UserAlreadySubscribed
        );

	});
}

#[test]
fn register_service_to_unknown_service_provider() {
	new_test_ext().execute_with(|| {
        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service+1, period, receiver_account, fee));

        assert_noop!(
		    ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+1, service, period, receiver_account, fee),
            Error::<Test>::ServiceProviderNotRegistered
        );
    });
}

#[test]
fn user_subscribes_to_unknown_service_fails() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);
        Balances::make_free_balance_be(&2,99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));

        assert_ok!(
            ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service),
            ()
        );

        assert_noop!(
            ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service+1),
            Error::<Test>::ServiceNotKnown
        );
	});
}

#[test]
fn user_subscribes_to_unknown_service_provider_fails() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);
        Balances::make_free_balance_be(&2,99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));

        assert_ok!(
            ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service),
            ()
        );

        assert_noop!(
            ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider+1, service),
            Error::<Test>::ServiceProviderNotRegistered
        );
	});
}

#[test]
fn user_subscribes_and_cancels() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);
        Balances::make_free_balance_be(&2,4*99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider+1));

		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service+1, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+1, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider+1, service+1, period, receiver_account, fee));

		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service));
        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(11);
		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service+1));
        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(12);
		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider+1, service));
        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(13);
		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider+1, service+1));

		assert_ok!(
            ServiceSubscriptionModule::cancel(Origin::signed(2), service_provider+1, service+1),
            ()
        );

        let vb : UserSubscriptionsVec = bounded_vec![20,21,22];
        assert_eq!(pallet_service_subscription::pallet::UserSubscriptions::<Test>::get(2), vb);

	});
}

#[test]
fn cancel_to_unsubscribed_fails() {
	new_test_ext().execute_with(|| {

        let service_provider = 42;
        let service = 1;
        let fee = 99;
        let period = 10;
        let receiver_account = 1;

        <ServiceSubscriptionModule as OnInitialize<<Test as system::Config>::BlockNumber>>::on_initialize(10);
        Balances::make_free_balance_be(&2,99);

		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), service_provider));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service, period, receiver_account, fee));
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), service_provider, service+1, period, receiver_account, fee));

		assert_ok!(ServiceSubscriptionModule::subscribe(Origin::signed(2), service_provider, service));

		assert_noop!(
            ServiceSubscriptionModule::cancel(Origin::signed(2), service_provider, service+1),
            Error::<Test>::UserNotSubscribed
        );
	});
}
