use near_sdk::{require, Promise};

use crate::*;

impl Contract {
    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, Balance)> {
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        if let Some(balance) = self.accounts.get(&account_id) {
            if balance == 0 || force {
                self.accounts.remove(&account_id);
                self.total_supply -= balance;
                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
                Some((account_id, balance))
            } else {
                env::panic_str(
                    "Can't unregister the account with the positive balance without force",
                )
            }
        } else {
            log!("The account {} is not registered", &account_id);
            None
        }
    }

    pub fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        if self.accounts.contains_key(account_id) {
            Some(StorageBalance { total: self.storage_balance_bounds().min, available: 0.into() })
        } else {
            None
        }
    }

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        match self.accounts.get(account_id) {
            Some(balance) => balance,
            None => {
                env::panic_str(format!("The account {} is not registered", &account_id).as_str())
            }
        }
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_add(amount) {
            self.accounts.insert(account_id, &new_balance);
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("Balance overflow");
        }
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.accounts.insert(account_id, &new_balance);
            self.total_supply = self
                .total_supply
                .checked_sub(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("The account doesn't have enough balance");
        }
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        require!(sender_id != receiver_id, "Sender and receiver should be different");
        require!(amount > 0, "The amount should be a positive number");
        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);
        FtTransfer {
            old_owner_id: sender_id,
            new_owner_id: receiver_id,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
        .emit();
    }

    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self.accounts.insert(account_id, &0).is_some() {
            env::panic_str("The account is already registered");
        }
    }

    pub fn measure_bytes_for_longest_account_id(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = AccountId::new_unchecked("a".repeat(64));
        self.accounts.insert(&tmp_account_id, &0u128);
        self.bytes_for_longest_account_id = env::storage_usage() - initial_storage_usage;
        self.accounts.remove(&tmp_account_id);
    }
}