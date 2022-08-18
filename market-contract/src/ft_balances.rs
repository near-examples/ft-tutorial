use near_sdk::require;

use crate::*;

/// transfer callbacks from FT Contracts

/*
    trait that will be used as the callback from the FT contract. When ft_transfer_call is
    called, it will fire a cross contract call to this marketplace and this is the function
    that is invoked. 
*/
trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> U128;

    fn ft_withdraw(
        &mut self,
        amount: U128
    );

    fn resolve_refund(
        &mut self,
        amount: U128,
        caller: AccountId
    );
}

//implementation of the trait
#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// This is how users will fund their FT balances in the contract
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> U128 {
        // get the contract ID which is the predecessor
        let ft_contract_id = env::predecessor_account_id();
        // Ensure only the specified FT can be used
        require!(
            ft_contract_id == self.ft_id,
            "FT contract ID does not match"
        );
        
        //get the signer which is the person who initiated the transaction
        let signer_id = env::signer_account_id();

        //make sure that the signer isn't the predecessor. This is so that we're sure
        //this was called via a cross-contract call
        assert_ne!(
            ft_contract_id,
            signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        //make sure the owner ID is the signer. 
        assert_eq!(
            sender_id,
            signer_id,
            "owner_id should be signer_id"
        );

        // Add the amount to the user's current balance
        let cur_bal = self.ft_deposits.get(&signer_id).unwrap_or(0);
        cur_bal += amount.0;
        self.ft_deposits.insert(&signer_id, &cur_bal);

        // We don't return any FTs to the sender because we're storing all of them in their balance
        U128(0)
    }

    #[payable]
    fn ft_withdraw(
            &mut self,
            amount: U128
    ) {
        //make sure the user attaches exactly 1 yoctoNEAR for security purposes.
        //this will redirect them to the NEAR wallet (or requires a full access key). 
        assert_one_yocto();

        // Get the caller and ensure they have enough balance
        let caller = env::predecessor_account_id();
        let cur_bal = self.ft_deposits.get(&caller).unwrap_or(0);
        require!(
            cur_bal >= amount.0,
            "Insufficient balance"
        );

        // Subtract the amount from the caller's balance
        let new_bal = cur_bal - amount.0;
        self.ft_deposits.insert(&caller, &new_bal);

        // Perform the cross contract call to transfer the FTs to the caller. If anything goes wrong
        // We increment their balance back when we resolve the promise
        ext_ft_contract::ext(self.ft_id)
            // Attach 1 yoctoNEAR with static GAS equal to the GAS for nft transfer. Also attach an unused GAS weight of 1 by default.
            .with_attached_deposit(1)
            .ft_transfer(
                caller, //caller to refund the FTs to
                amount, //amount to transfer
                Some("Withdrawing from Marketplace".to_string()), //memo (to include some context)
            )
        .then(
            // No attached deposit with static GAS equal to the GAS for resolving the purchase. Also attach an unused GAS weight of 1 by default.
            Self::ext(env::current_account_id())
            .with_static_gas(GAS_FOR_RESOLVE_REFUND)
            .resolve_refund(
                caller, //caller to refund the FTs to
                amount, //amount to transfer
            )
        );
    }
}