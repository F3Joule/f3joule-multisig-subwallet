use rstd::prelude::*;
use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, ensure, dispatch::Result, Parameter};
use parity_codec::Codec;
use parity_codec::{Encode, Decode};
use runtime_primitives::traits::{As};
use rstd::collections::btree_map::BTreeMap;
use primitives::{sr25519, Pair};
use {balances, timestamp};
use system::{self, ensure_signed};

pub const MIN_MULTISIG_WALLET_OWNERS: u16 = 2;
pub const MAX_MULTISIG_WALLET_OWNERS: u16 = 15;
pub const MAX_NOTES_LEN: u16 = 128;

pub const MSG_NOT_ENOUGH_OWNERS: &str = "There can not be less owners than allowed";
pub const MSG_TOO_MANY_OWNERS: &str = "There can not be more owners than allowed";
pub const MSG_MORE_REQUIRES_THAN_OWNERS: &str = "The required confirmation count can not be greater than owners count";
pub const MSG_WALLET_BY_ACCOUNT_NOT_FOUND: &str = "Account don't owe multi-signature wallet";
pub const MSG_WALLET_NOT_FOUND: &str = "Multi-signature wallet not found by id";
pub const MSG_NOT_A_WALLET_OWNER: &str = "Account is not a wallet owner";
pub const MSG_TRANSACTION_NOTES_TOO_LONG: &str = "Transaction notes are too long";
pub const MSG_OVERFLOW_CREATING_TRANSACTION: &str = "Transactions count overflow creating new transaction";
// pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";
// pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";
// pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";
// pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";
// pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";
// pub const MSG_ACCOUNT_CANNOT_FOLLOW_ITSELF: &str = "Account can not follow itself";

#[derive(Encode, Decode)]
pub struct Change<T: Trait> {
	account: T::AccountId,
	block: T::BlockNumber,
	time: T::Moment,
}

#[derive(Encode, Decode)]
pub struct Wallet<T: Trait> {
	created: Change<T>,
	id: T::AccountId,
	owners: Vec<T::AccountId>,
	confirms_required: u16,
}

#[derive(Encode, Decode)]
pub struct Transaction<T: Trait> {
	created: Change<T>,
	destination: T::AccountId,
	value: T::Balance,
	notes: Vec<u8>,
	confirmed_by: Vec<T::AccountId>,
	executed: bool,
}

pub trait Trait: system::Trait + balances::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as MultisigWalletModule {
		MinMultisigWalletOwners get(min_multisig_wallet_owners): u16 = MIN_MULTISIG_WALLET_OWNERS;
		MaxMultisigWalletOwners get(max_multisig_wallet_owners): u16 = MAX_MULTISIG_WALLET_OWNERS;

		WalletById get(wallet_by_id): map T::AccountId => Option<Wallet<T>>;
		WalletIdsByAccountId get(wallet_ids_by_account_id): map T::AccountId => Vec<T::AccountId>;

		TransactionIdsByWalletId get(transaction_ids_by_wallet_id): map T::AccountId => Vec<Transaction<T>>;
		TransactionsByWalletId get(transactions_by_wallet_id): map T::AccountId => u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		fn create_wallet(origin, owners: Vec<T::AccountId>, confirms_required: u16) -> Result {
			let creator = ensure_signed(origin)?;
			let mut owners_map: BTreeMap<T::AccountId, bool> = BTreeMap::new();
			let mut wallet_owners: Vec<T::AccountId> = vec![];

			ensure!(owners.len() >= MIN_MULTISIG_WALLET_OWNERS as usize, MSG_NOT_ENOUGH_OWNERS);
			ensure!(owners.len() <= MAX_MULTISIG_WALLET_OWNERS as usize, MSG_TOO_MANY_OWNERS);

			for owner in owners.iter() {
				if !owners_map.contains_key(&owner) {
					wallet_owners.push(*owner);
				}
			}

			ensure!(confirms_required as usize <= owners.len(), MSG_MORE_REQUIRES_THAN_OWNERS);

			let pair = sr25519::Pair::generate();
			let wallet_id: T::AccountId = T::AccountId::from(pair.public());
			let new_wallet = Wallet {
				created: Self::new_change(creator.clone()),
				id: wallet_id,
				owners: wallet_owners,
				confirms_required
			};

			<WalletById<T>>::insert(wallet_id.clone(), new_wallet);
			<WalletIdsByAccountId<T>>::mutate(creator.clone(), |ids| ids.push(wallet_id.clone()));
			<TransactionsByWalletId<T>>::insert(wallet_id.clone(), 0);

			Self::deposit_event(RawEvent::WalletCreated(creator, wallet_id));

			Ok(())
		}

		fn create_transaction(origin, wallet_id: T::AccountId, destination: T::AccountId, value: T::Balance, notes: Vec<u8>) -> Result {
			let creator = ensure_signed(origin)?;

			ensure!(notes.len() <= MAX_NOTES_LEN as usize, MSG_TRANSACTION_NOTES_TOO_LONG);
			let wallet = Self::wallet_by_id(wallet_id.clone()).ok_or(MSG_WALLET_NOT_FOUND)?;
			ensure!(wallet.owners.iter().any(|&x| x == creator.clone()), MSG_NOT_A_WALLET_OWNER);

			let new_transactions_count = Self::transactions_by_wallet_id(wallet_id.clone()).checked_add(1).ok_or(MSG_OVERFLOW_CREATING_TRANSACTION)?;
			let mut new_transaction = Transaction {
				created: Self::new_change(creator.clone()),
				destination,
				value,
				notes,
				confirmed_by: vec![],
				executed: false
			};

			new_transaction.confirmed_by.push(creator.clone());

			<TransactionIdsByWalletId<T>>::mutate(wallet_id.clone(), |ids| ids.push(new_transaction));
			<TransactionsByWalletId<T>>::insert(wallet_id.clone(), new_transactions_count);

			Self::deposit_event(RawEvent::TransactionCreated(creator, wallet_id, destination, value));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as balances::Trait>::Balance
	{
		WalletCreated(AccountId, AccountId),
		TransactionCreated(AccountId, AccountId, AccountId, Balance),
		// TransactionSubmitted(AccountId, AccountId, AccountId, Balance),
	}
);

impl<T: Trait> Module<T> {
	fn new_change(account: T::AccountId) -> Change<T> {
    Change {
      account,
      block: <system::Module<T>>::block_number(),
      time: <timestamp::Module<T>>::now(),
    }
  }
}

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
	type MultisigWalletModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(MultisigWalletModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(MultisigWalletModule::something(), Some(42));
		});
	}
}
