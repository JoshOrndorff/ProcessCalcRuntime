/// A runtime module that interprets a very simple process calculus
///
/// Grammar
/// P ::= Send
///     | Recv

use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;
use parity_codec::{ Encode, Decode };

/// All the types of processes in our calculus
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
pub enum Proc {
    //TODO make them parametric in a channel
    Send,
    Receive,
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// Idea: Maybe make the module parametric in the process type

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// The tuplespace
decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		// How many sends are stored in the tuplespace
        Sends get(num_sends): u32;

        // How many receives are stored in the tuplespace
        Receives get(num_receives): u32;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		// Deploy a term into the tuplespace
		pub fn deploy(origin, term: Proc) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let deployer = ensure_signed(origin)?;

			match term {
                Proc::Send => {
                    // Add the term to the storage
                    //TODO check for overflow and emit an error
                    // Gav wrote in riot
                    // .saturated_into or .checked_into; there are traits in srml_primitives to help you move between numeric types.
                    <Sends<T>>::mutate(|n| *n += 1);

                    //Emit and event
                    Self::deposit_event(RawEvent::Deployed(deployer, Proc::Send));
                }

                Proc::Receive => {
                    <Receives<T>>::mutate(|n| *n += 1);
                    Self::deposit_event(RawEvent::Deployed(deployer, Proc::Receive));
                }
            }

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        //TODO I don't really care who deployed this, but I couldn't
        // make the typechecker happy when I didn't use AccountId
		// Event fires when any term is deployed to the tuplespace
		Deployed(AccountId, Proc),
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
