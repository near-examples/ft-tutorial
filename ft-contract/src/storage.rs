use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, AccountId, Balance, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
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

    // Withdraw specified amount of available Ⓝ for predecessor account.
    //
    // This method is safe to call. It MUST NOT remove data.
    //
    // `amount` is sent as a string representing an unsigned 128-bit integer. If
    // omitted, contract MUST refund full `available` balance. If `amount` exceeds
    // predecessor account's available balance, contract MUST panic.
    //
    // If predecessor account not registered, contract MUST panic.
    //
    // MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    // function-call access-key call (UX wallet security)
    //
    // Returns the StorageBalance structure showing updated balances.
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance;

    // Unregisters the predecessor account and returns the storage NEAR deposit.
    //
    // If the predecessor account is not registered, the function MUST return
    // `false` without panic.
    //
    // If `force=true` the function SHOULD ignore existing account data, such as
    // non-zero balances on an FT contract (that is, it should burn such balances),
    // and close the account. Contract MAY panic if it doesn't support forced
    // unregistration, or if it can't force unregister for the particular situation
    // (example: too much data to delete at once).
    //
    // If `force=false` or `force` is omitted, the contract MUST panic if caller
    // has existing account data, such as a positive registered balance (eg token
    // holdings).
    //
    // MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    // function-call access-key call (UX wallet security)
    //
    // Returns `true` iff the account was successfully unregistered.
    // Returns `false` iff account was not registered before.
    fn storage_unregister(&mut self, force: Option<bool>) -> bool;

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
        let amount: Balance = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        if self.accounts.contains_key(&account_id) {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic_str("The attached deposit is less than the minimum storage balance");
            }

            self.internal_register_account(&account_id);
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }

    /// While storage_withdraw normally allows the caller to retrieve `available` balance, the basic
    /// Fungible Token implementation sets storage_balance_bounds.min == storage_balance_bounds.max,
    /// which means available balance will always be 0. So this implementation:
    /// * panics if `amount > 0`
    /// * never transfers Ⓝ to caller
    /// * returns a `storage_balance` struct if `amount` is 0
    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic_str("The amount is greater than the available storage balance");
                }
                _ => storage_balance,
            }
        } else {
            env::panic_str(
                format!("The account {} is not registered", &predecessor_account_id).as_str(),
            );
        }
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        self.internal_storage_unregister(force).is_some()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.bytes_for_longest_account_id) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.internal_storage_balance_of(&account_id)
    }
}
