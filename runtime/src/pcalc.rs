/// A runtime module that interprets a very simple process calculus
///
/// Grammar
/// P ::= Send
///     | Recv


// TODO items
// [check] Dispatchable call to create a comm reduction
// [check] Event for reductions
// Give each send or receive a unique ID
// Add the Nil Process
// Comms specify _which_ terms are being reduced
// Terms have continuations
// Terms are parametric in a channel (commed terms must be over same channel)
// Channels can be public or unforgeable

use support::{decl_module, decl_storage, decl_event, StorageMap, dispatch::Result, ensure};
use system::ensure_signed;
use parity_codec::{ Encode, Decode };

/// All the types of processes in our calculus
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub enum Proc {
    //TODO make them parametric in a channel
    Send,
    Receive,
}

//TODO why did we need a default here?
//TODO Make Nil the default if we do indeed need a default
impl Default for Proc {
    fn default() -> Self {
        Proc::Send
    }
}

//TODO add this to the configuration trait
type ProcId = u32;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// Idea: Maybe make the module parametric in the process type

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// The tuplespace
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// How many sends are stored in the tuplespace
        Sends get(sends): map ProcId => Proc;

        // How many receives are stored in the tuplespace
        Receives get(receives): map ProcId => Proc;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		// Deploy a term into the tuplespace
        //TODO eventually we should choose IDs pseudorandomly not take them
        // from the user
		pub fn deploy(origin, id: ProcId, term: Proc) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let deployer = ensure_signed(origin)?;

			match term {
                Proc::Send => {
                    // Add the term to the storage
                    <Sends<T>>::insert(id, term);

                    //Emit and event
                    Self::deposit_event(RawEvent::Deployed(deployer, id, Proc::Send));
                }

                Proc::Receive => {
                    <Receives<T>>::insert(id, term);
                    Self::deposit_event(RawEvent::Deployed(deployer, id, Proc::Receive));
                }
            }

			Ok(())
		}

        pub fn comm(origin, send: ProcId, receive: ProcId) -> Result {
            // Ensure the transaction was signed. (Might not be necessary)
            let _ = ensure_signed(origin)?;

            // Ensure that the specified send exists
            ensure!(<Sends<T>>::exists(send), "No such send in the tuplespace to be commed");

            // Ensure there is at least one receive
            ensure!(<Receives<T>>::exists(receive), "No such receive in the tuplespace to be commed");

            // Consume both
            <Sends<T>>::remove(send);
            <Receives<T>>::remove(receive);

            // TODO re-deploy the continuation

            // Emit the event
            Self::deposit_event(RawEvent::Comm(send, receive));

            Ok(())
        }
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        //TODO I don't really care who deployed this, but I couldn't
        // make the typechecker happy when I didn't use AccountId
		// Event fires when any term is deployed to the tuplespace
		Deployed(AccountId, ProcId, Proc),

        // Send then Receive
        Comm(ProcId, ProcId),
	}
);

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type ProcessCalc = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn deploying_a_send_works() {
		with_externalities(&mut new_test_ext(), || {
			// Deploy a single send
			assert_ok!(ProcessCalc::deploy(Origin::signed(1), Proc::Send));
			// Asserting that the send count increased
			assert_eq!(ProcessCalc::num_sends(), 1);
		});
	}
}
