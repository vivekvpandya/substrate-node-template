use crate::mock::*;
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		Scheduler::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Scheduler::on_initialize(System::block_number());
	}
}
#[test]
fn tree_test() {
	new_test_ext().execute_with(|| {
		run_to_block(1);
		let call = Box::new(Call::Tree(pallet_tree::Call::<Test>::cleanup {}.into()));
		assert_ok!(GarbageCollecter::schedule_cleanup(
			frame_system::RawOrigin::Root.into(),
			1,
			10,
			call
		));
		run_to_block(11);
		// Proofs that indeed clenup on tree pallet has triggered
		System::assert_has_event(Event::Tree(pallet_tree::Event::HeightDecreased(0)));
		let height = Tree::height();
		// Tree has initial height of 5, and due to cleanup it will be 0
		assert_eq!(height, 0);
	});
}
