use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
 use frame_support::instances::{Instance1, Instance2};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(TemplateModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn set_balance_works() {
	new_test_ext().execute_with(|| {
		let alice = 0u64;
		//let alice_balance = <Test as crate::Config>::Currency::free_balance(&alice);

		// in crate::Config we expect "something" which implements Currency
		// in mock, we set crate::Config::Currency to "Balances"
		// in pallet_balances::Config we set accountstore to "store in balances"


		let alice_balance = pallet_balances::pallet::Account::<Test, Instance1>::get(&alice).free;
		//let alice_balance = Balances::free_balance(&alice);
		assert_eq!(alice_balance, 0);
		assert_ok!(TemplateModule::set_balance(Origin::root(), alice, 1337, 6969));

		//let alice_new_balance = <Test as crate::Config>::Currency::free_balance(&alice);
		//let alice_new_balance = pallet_balances::pallet::Account::<Test, Instance1>::get(&alice).free;
		//let alice_new_balance = Balances::free_balance(&alice);
		let alice_new_balance = pallet_balances::Pallet::<Test, Instance1>::free_balance(&alice);
		let alice_new_balance2 = pallet_balances::Pallet::<Test, Instance2>::free_balance(&alice);

		assert_eq!(alice_new_balance, 1337);
		assert_eq!(alice_new_balance2, 6969);

	});
}

