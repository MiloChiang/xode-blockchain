use crate::{
	mock::*, 
	ProposedCandidates,
	Status,
	NextBlockNumber,
};
use frame_support::{
	assert_ok,
};
use frame_support::traits::{
	Currency, 
	Imbalance,
	Hooks,
};
use pallet_session::SessionManager;

// https://github.com/Xode-DAO/xode-blockchain/issues/111
// https://github.com/Xode-DAO/xode-blockchain/issues/110
// Test the bond_candidate
#[test]
fn test_pallet_xode_staking_last_updated_and_incorrect_balance_works() {
	test1_ext().execute_with(|| {
		System::set_block_number(0);
		XodeStaking::on_initialize(System::block_number());

        let candidate = 1;
		assert_ok!(XodeStaking::register_candidate(RuntimeOrigin::signed(candidate)));

		// Scenario 0: when the current bond is zero
		let imbalance = Balances::deposit_creating(&candidate, 100_000_000_000_000_000);
		assert!(imbalance.peek() > 0, "Expected a positive imbalance for deposit creation");

		assert_ok!(XodeStaking::bond_candidate(RuntimeOrigin::signed(candidate), 11_000_000_000_000_000));	
		assert_eq!(89_000_000_000_000_000, Balances::free_balance(&candidate), "Must match");

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");	

		assert_eq!(proposed_candidates[0].last_updated, 0, "Must match");
		assert_eq!(proposed_candidates[0].bond, 11_000_000_000_000_000u128, "Must match");

		// Scenario 1: when the bond is lesser than the current bond
		System::set_block_number(1);
		XodeStaking::on_initialize(System::block_number());

		assert_ok!(XodeStaking::bond_candidate(RuntimeOrigin::signed(candidate), 5_000_000_000_000_000));
		assert_eq!(95_000_000_000_000_000, Balances::free_balance(&candidate), "Must match");

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");	

		assert_eq!(proposed_candidates[0].last_updated, 1, "Must match");
		assert_eq!(proposed_candidates[0].bond, 5_000_000_000_000_000u128, "Must match");

		// Scenario 2: when the new bond is greater than the current bond
		System::set_block_number(2);
		XodeStaking::on_initialize(System::block_number());

		assert_ok!(XodeStaking::bond_candidate(RuntimeOrigin::signed(candidate), 15_000_000_000_000_000));
		assert_eq!(85_000_000_000_000_000, Balances::free_balance(&candidate), "Must match");

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");	

		assert_eq!(proposed_candidates[0].last_updated, 2, "Must match");
		assert_eq!(proposed_candidates[0].bond, 15_000_000_000_000_000u128, "Must match");

		// Scenario 3: when the new bond is equal to the current bond
		System::set_block_number(3);
		XodeStaking::on_initialize(System::block_number());

		assert_ok!(XodeStaking::bond_candidate(RuntimeOrigin::signed(candidate), 15_000_000_000_000_000));
		assert_eq!(85_000_000_000_000_000, Balances::free_balance(&candidate), "Must match");

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");	

		assert_eq!(proposed_candidates[0].last_updated, 2, "Must match");
		assert_eq!(proposed_candidates[0].bond, 15_000_000_000_000_000u128, "Must match");
	});
}

// https://github.com/Xode-DAO/xode-blockchain/issues/109
// Test the queue_authors - a helper function that is called at the end of every session
#[test]
fn test_pallet_xode_staking_assignment_operator_works() {
	test1_ext().execute_with(|| {
		System::set_block_number(0);
		XodeStaking::on_initialize(System::block_number());

        let candidate = 1;
		assert_ok!(XodeStaking::register_candidate(RuntimeOrigin::signed(candidate)));

		let imbalance = Balances::deposit_creating(&candidate, 100_000_000_000_000_000);
		assert!(imbalance.peek() > 0, "Expected a positive imbalance for deposit creation");

		assert_ok!(XodeStaking::bond_candidate(RuntimeOrigin::signed(candidate), 11_000_000_000_000_000));	
		assert_eq!(89_000_000_000_000_000, Balances::free_balance(&candidate), "Must match");

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		

		// The initial status must be online.  This status is set when you register a candidate
		assert_eq!(proposed_candidates[0].status, Status::Online, "Must match");

		System::set_block_number((1 * MINUTES).into());
		XodeStaking::on_initialize(System::block_number());

		XodeStaking::new_session(1);
		Session::on_initialize(System::block_number()); 

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		

		// Bonded online candidate will be automatically wait listed
		assert_eq!(proposed_candidates[0].status, Status::Waiting, "Must match");

		System::set_block_number((2 * MINUTES).into());
		XodeStaking::on_initialize(System::block_number());

		XodeStaking::new_session(2);
		Session::on_initialize(System::block_number()); 

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		

		// Wait listed candidate at the beginning of the session will be automatically listed as authoring
		assert_eq!(proposed_candidates[0].status, Status::Authoring, "Must match");		

		// Offline the candidate
		let _ = XodeStaking::offline_candidate(RuntimeOrigin::signed(candidate));

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		
		assert_eq!(proposed_candidates[0].offline, true, "Must match");			

		System::set_block_number((3 * MINUTES).into());
		XodeStaking::on_initialize(System::block_number());

		XodeStaking::new_session(3);
		Session::on_initialize(System::block_number()); 

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		

		// It takes two session before the author is transferred to queuing
		assert_eq!(proposed_candidates[0].status, Status::Authoring, "Must match");			

		System::set_block_number((4 * MINUTES).into());
		XodeStaking::on_initialize(System::block_number());

		XodeStaking::new_session(4);
		Session::on_initialize(System::block_number()); 

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		

		// After two sessions the author is now transferred to queuing
		assert_eq!(proposed_candidates[0].status, Status::Queuing, "Must match");				

		// Online the candidate
		let _ = XodeStaking::online_candidate(RuntimeOrigin::signed(candidate));

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 1, "The number of proposed candidates should be 1");		
		assert_eq!(proposed_candidates[0].offline, false, "Must match");		

		// Additional candidates, we need to maximized the proposed candidates so that the remaining
		// waiting candidates that exceeds the maximum allowed candidates will be queued.  We only put
		// 99 candidates to accommodate the existing candidate.
		for i in 2..101 {
			let _ = Balances::deposit_creating(&i, 100_000_000_000_000_000);
			assert_ok!(XodeStaking::register_candidate(RuntimeOrigin::signed(i)));
			assert_ok!(XodeStaking::bond_candidate(RuntimeOrigin::signed(i), 11_000_000_000_000_000));	
		}		

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 100, "The number of proposed candidates should be 100");	

		for i in 1..100 {
			if i == 1 {
				assert_eq!(proposed_candidates[i-1].status, Status::Queuing, "Must match");
			} else {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			}
		}		

		System::set_block_number((5 * MINUTES).into());
		XodeStaking::on_initialize(System::block_number());

		XodeStaking::new_session(5);
		Session::on_initialize(System::block_number()); 		

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 100, "The number of proposed candidates should be 100");	

		// Three candidate remains online because of the three Xaver nodes inserted.
		for i in 1..100 {
			if i == 1 {
				assert_eq!(proposed_candidates[i-1].status, Status::Queuing, "Must match");
			} else if i == 98 {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			} else if i == 99 {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			} else if i == 100 {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			} else {
				assert_eq!(proposed_candidates[i-1].status, Status::Waiting, "Must match");
			}
		}			

		System::set_block_number((6 * MINUTES).into());
		XodeStaking::on_initialize(System::block_number());

		XodeStaking::new_session(6);
		Session::on_initialize(System::block_number()); 		

		let proposed_candidates = ProposedCandidates::<Test>::get();
		assert_eq!(proposed_candidates.len(), 100, "The number of proposed candidates should be 100");	

		// The queued author will immediately author because it is still listed in the waiting list.
		// Converting queued author to waiting status at the same time set to author will happen
		// simultaneously because the author is already in the waiting list.
		for i in 1..100 {
			if i == 1 {
				assert_eq!(proposed_candidates[i-1].status, Status::Authoring, "Must match");
			} else if i == 98 {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			} else if i == 99 {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			} else if i == 100 {
				assert_eq!(proposed_candidates[i-1].status, Status::Online, "Must match");
			} else {
				assert_eq!(proposed_candidates[i-1].status, Status::Authoring, "Must match");
			}
		}					
	});
}

// https://github.com/Xode-DAO/xode-blockchain/issues/108
#[test]
fn test_pallet_xode_staking_on_initialize_weight_works() {
	test1_ext().execute_with(|| {
		System::set_block_number(0);
		let weight = XodeStaking::on_initialize(System::block_number());

		// Heavy weight is called only when the next block is explicitly cleared which only
		// happens at block 0.

		let expected = <Test as frame_system::Config>::DbWeight::get().reads_writes(14, 28);
        assert_eq!(weight, expected);		

		// To test if there is a next block which usually the setup on every Substrate chain.

		NextBlockNumber::<Test>::put(2);

		System::set_block_number(1);
		let weight = XodeStaking::on_initialize(System::block_number());

		let expected = <Test as frame_system::Config>::DbWeight::get().reads_writes(2, 2);
        assert_eq!(weight, expected);		
	});
}
