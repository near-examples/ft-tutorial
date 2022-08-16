use near_sdk::{Gas, ext_contract, PromiseOrValue, assert_one_yocto, PromiseResult};

use crate::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

#[ext_contract(ext_ft_core)]
pub trait FungibleTokenCore {
    /// Transfers positive `amount` of tokens from the `env::predecessor_account_id` to `receiver_id`.
    /// Both accounts must be registered with the contract for transfer to succeed. (See [NEP-145](https://github.com/near/NEPs/discussions/145))
    /// This method must to be able to accept attached deposits, and must not panic on attached deposit.
    /// Exactly 1 yoctoNEAR must be attached.
    /// See [the Security section](https://github.com/near/NEPs/issues/141#user-content-security) of the standard.
    ///
    /// Arguments:
    /// - `receiver_id` - the account ID of the receiver.
    /// - `amount` - the amount of tokens to transfer. Must be a positive number in decimal string representation.
    /// - `memo` - an optional string field in a free form to associate a memo with this transfer.
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);

    /// Transfers positive `amount` of tokens from the `env::predecessor_account_id` to `receiver_id` account. Then
    /// calls `ft_on_transfer` method on `receiver_id` contract and attaches a callback to resolve this transfer.
    /// `ft_on_transfer` method must return the amount of tokens unused by the receiver contract, the remaining tokens
    /// must be refunded to the `predecessor_account_id` at the resolve transfer callback.
    ///
    /// Token contract must pass all the remaining unused gas to the `ft_on_transfer` call.
    ///
    /// Malicious or invalid behavior by the receiver's contract:
    /// - If the receiver contract promise fails or returns invalid value, the full transfer amount must be refunded.
    /// - If the receiver contract overspent the tokens, and the `receiver_id` balance is lower than the required refund
    /// amount, the remaining balance must be refunded. See [the Security section](https://github.com/near/NEPs/issues/141#user-content-security) of the standard.
    ///
    /// Both accounts must be registered with the contract for transfer to succeed. (See #145)
    /// This method must to be able to accept attached deposits, and must not panic on attached deposit. Exactly 1 yoctoNEAR must be attached. See [the Security
    /// section](https://github.com/near/NEPs/issues/141#user-content-security) of the standard.
    ///
    /// Arguments:
    /// - `receiver_id` - the account ID of the receiver contract. This contract will be called.
    /// - `amount` - the amount of tokens to transfer. Must be a positive number in a decimal string representation.
    /// - `memo` - an optional string field in a free form to associate a memo with this transfer.
    /// - `msg` - a string message that will be passed to `ft_on_transfer` contract call.
    ///
    /// Returns a promise which will result in the amount of tokens withdrawn from sender's account.
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    /// Returns the total supply of the token in a decimal string representation.
    fn ft_total_supply(&self) -> U128;

    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        // Assert that the user attached exactly 1 yoctoNEAR. This is for security and so that the user will be required to sign with a FAK.
        assert_one_yocto();
        // The sender is the user who called the method
        let sender_id = env::predecessor_account_id();
        // How many tokens the user wants to withdraw
        let amount: Balance = amount.into();
        // Transfer the tokens
        self.internal_transfer(&sender_id, &receiver_id, amount, memo);
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Assert that the user attached exactly 1 yoctoNEAR. This is for security and so that the user will be required to sign with a FAK.
        assert_one_yocto();
        // The sender is the user who called the method
        let sender_id = env::predecessor_account_id();
        // How many tokens the sender wants to transfer
        let amount: Balance = amount.into();
        // Transfer the tokens
        self.internal_transfer(&sender_id, &receiver_id, amount, memo);

        // Initiating receiver's call and the callback
        // Defaulting GAS weight to 1, no attached deposit, and static GAS equal to the GAS for ft transfer call.
        ext_ft_receiver::ext(receiver_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER_CALL)
            .ft_on_transfer(sender_id.clone(), amount.into(), msg)
            // We then resolve the promise and call ft_resolve_transfer on our own contract
            // Defaulting GAS weight to 1, no attached deposit, and static GAS equal to the GAS for resolve transfer
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .ft_resolve_transfer(&sender_id, receiver_id, amount.into()),
            )
            .into()
    }

    fn ft_total_supply(&self) -> U128 {
        // Return the total supply casted to a U128
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        // Return the balance of the account casted to a U128
        self.accounts.get(&account_id).unwrap_or(0).into()
    }
}

#[ext_contract(ext_ft_receiver)]
pub trait FungibleTokenReceiver {
    /// Called by fungible token contract after `ft_transfer_call` was initiated by
    /// `sender_id` of the given `amount` with the transfer message given in `msg` field.
    /// The `amount` of tokens were already transferred to this contract account and ready to be used.
    ///
    /// The method must return the amount of tokens that are *not* used/accepted by this contract from the transferred
    /// amount. Examples:
    /// - The transferred amount was `500`, the contract completely takes it and must return `0`.
    /// - The transferred amount was `500`, but this transfer call only needs `450` for the action passed in the `msg`
    ///   field, then the method must return `50`.
    /// - The transferred amount was `500`, but the action in `msg` field has expired and the transfer must be
    ///   cancelled. The method must return `500` or panic.
    ///
    /// Arguments:
    /// - `sender_id` - the account ID that initiated the transfer.
    /// - `amount` - the amount of tokens that were transferred to this account in a decimal string representation.
    /// - `msg` - a string message that was passed with this transfer call.
    ///
    /// Returns the amount of unused tokens that should be returned to sender, in a decimal string representation.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl Contract {
    // Finalize an `ft_transfer_call` chain of cross-contract calls.
    //
    // The `ft_transfer_call` process:
    //
    // 1. Sender calls `ft_transfer_call` on FT contract
    // 2. FT contract transfers `amount` tokens from sender to receiver
    // 3. FT contract calls `ft_on_transfer` on receiver contract
    // 4+. [receiver contract may make other cross-contract calls]
    // N. FT contract resolves promise chain with `ft_resolve_transfer`, and may
    //    refund sender some or all of original `amount`
    //
    // Requirements:
    // * Contract MUST forbid calls to this function by any account except self
    // * If promise chain failed, contract MUST revert token transfer
    // * If promise chain resolves with a non-zero amount given as a string,
    //   contract MUST return this amount of tokens to `sender_id`
    //
    // Arguments:
    // * `sender_id`: the sender of `ft_transfer_call`
    // * `receiver_id`: the `receiver_id` argument given to `ft_transfer_call`
    // * `amount`: the `amount` argument given to `ft_transfer_call`
    //
    // Returns a string representing a string version of an unsigned 128-bit
    // integer of how many total tokens were spent by sender_id. Example: if sender
    // calls `ft_transfer_call({ "amount": "100" })`, but `receiver_id` only uses
    // 80, `ft_on_transfer` will resolve with `"20"`, and `ft_resolve_transfer`
    // will return `"80"`.
    pub fn ft_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            // If the promise was successful, get the return value and cast it to a U128.
            PromiseResult::Successful(value) => {
                // If we can properly parse the value, the unused amount is equal to whatever is smaller - the unused amount or the original amount (to prevent malicious contracts)
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                // If we can't properly parse the value, the original amount is returned.
                } else {
                    amount
                }
            }
            // If the promise wasn't successful, return the original amount.
            PromiseResult::Failed => amount,
        };

        // If there is some unused amount, we should refund the sender
        if unused_amount > 0 {
            // Get the receiver's balance. We can only refund the sender if the receiver has enough balance.
            let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
            if receiver_balance > 0 {
                // The amount to refund is the smaller of the unused amount and the receiver's balance as we can only refund up to what the receiver currently has.
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                
                // Remove the refund amount from the receiver's balance.
                if let Some(new_receiver_balance) = receiver_balance.checked_sub(refund_amount) {
                    self.accounts.insert(&receiver_id, &new_receiver_balance);
                } else {
                    env::panic_str("The receiver account doesn't have enough balance");
                }

                // Get the sender's current balance
                let sender_balance = self.accounts.get(sender_id).unwrap();
                // Add the refund amount to the sender's balance
                if let Some(new_sender_balance) = sender_balance.checked_add(refund_amount) {
                    self.accounts.insert(sender_id, &new_sender_balance);
                } else {
                    env::panic_str("Sender balance overflow");
                }

                // Emit a transfer log event
                FtTransfer {
                    old_owner_id: &receiver_id,
                    new_owner_id: sender_id,
                    amount: &U128(refund_amount),
                    memo: Some("refund"),
                }
                .emit();
                
                // Return what was actually used (the amount sent - refund)
                let used_amount = amount
                    .checked_sub(refund_amount)
                    .unwrap_or_else(|| env::panic_str("Total supply overflow"));
                return used_amount.into();
            }
        }

        // If the unused amount is 0, return the original amount.
        amount.into()
    }
}
