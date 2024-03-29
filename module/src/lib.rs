#![cfg_attr(not(feature = "std"), no_std)]

/// A runtime module that interprets a sipmlified pi calculus


use support::{decl_module, decl_storage, decl_event, StorageMap, dispatch::Result, ensure};
use system::ensure_signed;
use rstd::boxed::Box;
use codec::{ Encode, Decode };
use sr_primitives::traits::Hash;

/// All the types of processes in our calculus
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub enum Proc {
	Send(Channel),
	Receive(Channel, Box<Proc>),
	Nil,
}

// Need a default process because the sends and receives maps need
// to return a value when queried at non existant ids
impl Default for Proc {
	fn default() -> Self {
		Proc::Nil
	}
}

type ProcId<T> = <T as system::Trait>::Hash;
type Channel = u32;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// Idea: Maybe make the module parametric in the process type

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// The tuplespace
decl_storage! {
	trait Store for Module<T: Trait> as PCalc {
		// How many sends are stored in the tuplespace
		Sends get(sends): map ProcId<T> => Proc;

		// How many receives are stored in the tuplespace
		Receives get(receives): map ProcId<T> => Proc;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where
		origin: T::Origin
	{
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		// Deploy a term into the tuplespace
		//TODO eventually we should choose IDs pseudorandomly not take them
		// from the user
		pub fn deploy(origin, id: ProcId<T>, term: Proc) -> Result {
			// Deployer will be used to unlock unforgeables from locker room.
			let _deployer = ensure_signed(origin)?;

			Self::par_in(&term, id);

			Self::deposit_event(RawEvent::Deployed(id, term));

			Ok(())
		}

		pub fn comm(origin, send_id: ProcId<T>, receive_id: ProcId<T>) -> Result {
			// Ensure the transaction was signed. (Might not be necessary)
			let _ = ensure_signed(origin)?;

			// Ensure the specified send exists
			ensure!(<Sends<T>>::exists(send_id), "No such send in the tuplespace to be commed");

			// Ensure the specified receive exists
			ensure!(<Receives<T>>::exists(receive_id), "No such receive in the tuplespace to be commed");

			if let (Proc::Send(send_chan), Proc::Receive(receive_chan, continuation)) = (<Sends<T>>::get(send_id), <Receives<T>>::get(receive_id)) {
				// Ensure they are on the same channel
				ensure!(send_chan == receive_chan, "Send and receive must be on same channel");

				// Re-deploy the continuation
				let new_id = (send_id, receive_id).using_encoded(<T as system::Trait>::Hashing::hash);
				Self::par_in(&continuation, new_id);
			}

			// Consume both
			<Sends<T>>::remove(send_id);
			<Receives<T>>::remove(receive_id);

			// Emit the event
			Self::deposit_event(RawEvent::Comm(send_id, receive_id));

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	/// Pars the given term into the tuplespace at the given id
	fn par_in(term: &Proc, id: ProcId<T>) {
		match term {
			Proc::Send(_) => <Sends<T>>::insert(id, term),
			Proc::Receive(_, _) => <Receives<T>>::insert(id, term),
			Proc::Nil => (),
			// Recursive calls like par and new will go here.
			// When we have pars, we'll increment the id for each child.
		}
	}
}

decl_event!(
	pub enum Event<T> where ProcId = <T as system::Trait>::Hash {
		//TODO Why did I have to re-declare ProcId here?
		Deployed(ProcId, Proc),

		// Send then Receive
		Comm(ProcId, ProcId),
	}
);

// cargo test -p pcalc-runtime
/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::{H256, Blake2Hasher, Hasher};
	use support::{impl_outer_origin, assert_ok, assert_noop, parameter_types};
	use sr_primitives::{
		traits::{BlakeTwo256, IdentityLookup},
		testing::Header,
		weights::Weight,
		Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type ProcessCalc = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn deploying_a_send_works() {
		new_test_ext().execute_with(|| {
			let id: H256 = H256::from_low_u64_le(1);

			// Deploy a single send by user 1 with id 1 over channel 1
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id, Proc::Send(1)));
			// Assert that the send is in the tuplespace
			assert_eq!(<Sends<Test>>::get(id), Proc::Send(1));
		});
	}

	#[test]
	fn deploying_a_receive_works() {
		new_test_ext().execute_with(|| {
			let id: H256 = H256::from_low_u64_le(1);

			// Deploy a single receive by user 1 with id 1 over channel 1
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id, Proc::Receive(1, Box::new(Proc::Nil))));
			// Assert that the receive is in the tuplespace
			assert_eq!(<Receives<Test>>::get(id), Proc::Receive(1, Box::new(Proc::Nil)));
		});
	}

	#[test]
	fn comm_over_same_channel_works() {
		new_test_ext().execute_with(|| {
			let id1: H256 = H256::from_low_u64_le(1);
			let id2: H256 = H256::from_low_u64_le(2);
			let commed_id = (id1, id2).using_encoded(Blake2Hasher::hash);

			// Deploy send (id 1) and receive (id 2)
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id1, Proc::Send(1)));
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id2, Proc::Receive(1, Box::new(Proc::Send(2)))));

			// Run the comm event
			assert_ok!(ProcessCalc::comm(Origin::signed(1), id1, id2));

			// Assert both were consumed
			assert!(!<Sends<Test>>::exists(id1));
			assert!(!<Receives<Test>>::exists(id2));

			// Assert new term exists at expected id
			assert_eq!(<Sends<Test>>::get(commed_id), Proc::Send(2));
		});
	}

	#[test]
	fn comm_over_different_channels_fails() {
		new_test_ext().execute_with(|| {
			let id1: H256 = H256::from_low_u64_le(1);
			let id2: H256 = H256::from_low_u64_le(2);

			// Deploy send (chan 1) and receive (chan 2)
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id1, Proc::Send(1)));
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id2, Proc::Receive(2, Box::new(Proc::Nil))));

			// Assert that the comm event fails
			assert_noop!(ProcessCalc::comm(Origin::signed(1), id1, id2), "Send and receive must be on same channel");

			// Assert neither were consumed
			assert!(<Sends<Test>>::exists(id1));
			assert!(<Receives<Test>>::exists(id2));
		});
	}

	#[test]
	fn comm_with_missing_receive_fails() {
		new_test_ext().execute_with(|| {
			let id1: H256 = H256::from_low_u64_le(1);
			let id2: H256 = H256::from_low_u64_le(2);

			// Deploy send (id 1) but no receive
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id1, Proc::Send(1)));

			// Assert that the comm event fails
			assert_noop!(ProcessCalc::comm(Origin::signed(1), id1, id2), "No such receive in the tuplespace to be commed");

			// Assert send not consumed
			assert!(<Sends<Test>>::exists(id1));
		});
	}

	#[test]
	fn comm_with_missing_send_fails() {
		new_test_ext().execute_with(|| {
			let id1: H256 = H256::from_low_u64_le(1);
			let id2: H256 = H256::from_low_u64_le(2);

			// Deploy receive (id 2) but no send
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), id2, Proc::Receive(2, Box::new(Proc::Nil))));

			// Assert that the comm event fails
			assert_noop!(ProcessCalc::comm(Origin::signed(1), id1, id2), "No such send in the tuplespace to be commed");

			// Assert receive not consumed
			assert!(<Receives<Test>>::exists(id2));
		});
	}
}
