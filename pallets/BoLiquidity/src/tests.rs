use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// // Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), Some(42));

		BoLiquidityModule::create_lp(Origin::signed(1), String::from("Lp 1").as_bytes().to_vec(), 10000, 95);
		BoLiquidityModule::create_lp(Origin::signed(1), String::from("Lp 2").as_bytes().to_vec(), 600000, 95);
		

	});
}

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
// 	});
// }


/*
Test:
- [ ] same order data at the same timestamp, ... must created different order_id
 */
