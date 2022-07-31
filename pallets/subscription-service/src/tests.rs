use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn cannot_register_service_provider_twice() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_noop!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), Error::<Test>::ServiceProviderAlreadyRegistered);
	});
}


#[test]
fn register_single_service() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 0, 10, 1, 99), ());
	});
}

#[test]
fn register_two_services() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 0, 10, 1, 99), ());
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 1, 10, 1, 39), ());
	});
}

#[test]
fn fail_to_register_three_services() {
	new_test_ext().execute_with(|| {
		assert_ok!(ServiceSubscriptionModule::register_service_provider(Origin::signed(1), 42), ());
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 0, 10, 1, 99), ());
		assert_ok!(ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 1, 10, 1, 49), ());

        //2 services per service provider
		assert_noop!(ServiceSubscriptionModule::register_service(Origin::signed(1), 42, 2, 10, 1, 299), Error::<Test>::CannotRegisterService);
	});
}

