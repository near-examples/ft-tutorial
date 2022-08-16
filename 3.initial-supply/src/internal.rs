use near_sdk::{require};

use crate::*;

impl Contract {
    /// Internal method for depositing some amount of FTs into an account. 
    /// This will briefly increase the total supply but is always called in conjunction with `internal_withdraw` which decreases the total supply.
    /// The result is always a net 0 balance change. (this is only ever called on its own in the initialization function)
    pub(crate) fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        // Get the current balance of the account.
        let balance = self.accounts.get(&account_id).unwrap_or(0);
        
        // Add the amount to the balance and insert the new balance into the accounts map
        if let Some(new_balance) = balance.checked_add(amount) {
            self.accounts.insert(account_id, &new_balance);
            // Increment the total supply since we're depositing some FTs
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Total supply overflow"));
        } else {
            env::panic_str("Balance overflow");
        }
    }
}