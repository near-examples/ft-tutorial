use near_sdk::{env, log, AccountId, Promise};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::*;

// The structure that will be returned for the methods:
// * `storage_deposit`
// * `storage_withdraw`
// * `storage_balance_of`
// The `total` and `available` values are string representations of unsigned
// 128-bit integers showing the balance of a specific account in yoctoⓃ.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

// The below structure will be returned for the method `storage_balance_bounds`.
// Both `min` and `max` are string representations of unsigned 128-bit integers.
//
// `min` is the amount of tokens required to start using this contract at all
// (eg to register with the contract). If a new contract user attaches `min`
// NEAR to a `storage_deposit` call, subsequent calls to `storage_balance_of`
// for this user must show their `total` equal to `min` and `available=0` .
//
// A contract may implement `max` equal to `min` if it only charges for initial
// registration, and does not adjust per-user storage over time. A contract
// which implements `max` must refund deposits that would increase a user's
// storage balance beyond this amount.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

pub trait StorageManagement {
    /************************************/
    /* CHANGE METHODS on fungible token */
    /************************************/
    // Payable method that receives an attached deposit of Ⓝ for a given account.
    //
    // If `account_id` is omitted, the deposit MUST go toward predecessor account.
    // If provided, deposit MUST go toward this account. If invalid, contract MUST
    // panic.
    //
    // If `registration_only=true`, contract MUST refund above the minimum balance
    // if the account wasn't registered and refund full deposit if already
    // registered.
    //
    // The `storage_balance_of.total` + `attached_deposit` in excess of
    // `storage_balance_bounds.max` must be refunded to predecessor account.
    //
    // Returns the StorageBalance structure showing updated balances.
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    /****************/
    /* VIEW METHODS */
    /****************/
    // Returns minimum and maximum allowed balance amounts to interact with this
    // contract. See StorageBalanceBounds.
    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    // Returns the StorageBalance structure of the valid `account_id`
    // provided. Must panic if `account_id` is invalid.
    //
    // If `account_id` is not registered, must return `null`.
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
}

#[near_bindgen]
impl StorageManagement for Contract {
    #[allow(unused_variables)]
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        /*
            FILL THIS IN
        */
        todo!(); //remove once code is filled in.
    }
}
