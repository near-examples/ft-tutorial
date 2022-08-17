use near_sdk::{require};

use crate::*;

impl Contract {
    /// Internal method for depositing some amount of FTs into an account. 
    pub(crate) fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        // Get the current balance of the account.
        let balance = self.accounts.get(&account_id).unwrap_or(0);
        
        // Add the amount to the balance and insert the new balance into the accounts map
        if let Some(new_balance) = balance.checked_add(amount) {
            self.accounts.insert(account_id, &new_balance);
        } else {
            env::panic_str("Balance overflow");
        }
    }
}