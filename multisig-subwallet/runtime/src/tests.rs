#![cfg(test)]

use super::mock::*;
use super::multisigwallet::*;

use runtime_io::with_externalities;
use support::*;

const ACCOUNT1 : AccountId = 1;
const ACCOUNT2 : AccountId = 2;
const ACCOUNT3 : AccountId = 3;
const ACCOUNT4 : AccountId = 4;

fn default_walletid() -> AccountId {
  1
}

fn _create_default_wallet() -> dispatch::Result {
  _create_wallet(None, None, None, None, None)
}

fn _create_wallet(origin: Option<Origin>, wallet_id: Option<AccountId>, owners: Option<Vec<AccountId>>, max_tx_value: Option<CurrencyBalance>, confirms_required: Option<u16>) -> dispatch::Result {
  MultisigWallet::create_wallet(
    origin.unwrap_or(Origin::signed(ACCOUNT1)),
    wallet_id.unwrap_or(self::default_walletid()),
    owners.unwrap_or(vec![ACCOUNT1, ACCOUNT2, ACCOUNT3]),
    max_tx_value.unwrap_or(100),
    confirms_required.unwrap_or(2)
  )
}

#[test]
fn create_wallet_should_work() {
  with_externalities(&mut test_ext(), || {
    assert_ok!(_create_default_wallet());
  });
}