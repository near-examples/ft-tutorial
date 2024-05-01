use crate::*;
use near_sdk::PromiseResult;

//struct that holds important information about each sale on the market
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    //owner of the sale
    pub owner_id: AccountId,
    //market contract's approval ID to transfer the token on behalf of the owner
    pub approval_id: u32,
    //nft contract where the token was minted
    pub nft_contract_id: String,
    //actual token ID for sale
    pub token_id: String,
    //sale price in fungible tokens that the token is listed for
    pub sale_conditions: SalePriceInFTs,
}

#[near_bindgen]
impl Contract {
    
    //removes a sale from the market. 
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String) {
        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();
        //get the sale object as the return value from removing the sale internally
        let sale = self.internal_remove_sale(nft_contract_id.into(), token_id);
        //get the predecessor of the call and make sure they're the owner of the sale
        let owner_id = env::predecessor_account_id();
        //if this fails, the remove sale will revert
        assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
    }

    //updates the price for a sale on the market
    #[payable]
    pub fn update_price(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        price: U128,
    ) {
        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();
        
        //create the unique sale ID from the nft contract and token
        let contract_id: AccountId = nft_contract_id.into();
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        
        //get the sale object from the unique sale ID. If there is no token, panic. 
        let mut sale = self.sales.get(&contract_and_token_id).expect("No sale");

        //assert that the caller of the function is the sale owner
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );
        
        //set the sale conditions equal to the passed in price
        sale.sale_conditions = NearToken::from_yoctonear(price.0);
        //insert the sale back into the map for the unique sale ID
        self.sales.insert(&contract_and_token_id, &sale);
    }

    /// Place an offer on a specific sale. 
    /// The sale will go through as long as you have enough FTs in your balance to cover the amount and the amount is greater than or equal to the sale price
    #[payable]
    pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String, amount: U128) {
        let casted_amount = NearToken::from_yoctonear(amount.0);

        //assert that the user has attached exactly 1 yoctoNEAR (for security reasons)
        assert_one_yocto();

        //convert the nft_contract_id from a AccountId to an AccountId
        let contract_id: AccountId = nft_contract_id.into();
        //get the unique sale ID (contract + DELIMITER + token ID)
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        
        //get the sale object from the unique sale ID. If the sale doesn't exist, panic.
        let sale = self.sales.get(&contract_and_token_id).expect("No sale");
        
        //get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
        let buyer_id = env::predecessor_account_id();
        assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");
        
        //get the u128 price of the token
        let price = sale.sale_conditions;

        //make sure the amount offering is greater than or equal to the price of the token
        assert!(casted_amount.ge(&price), "Offer amount must be greater than or eqaul to the price: {:?}", price);

        // get the amount of FTs the buyer has in their balance
        let cur_bal = self.ft_deposits.get(&buyer_id).unwrap();
        //make sure the buyer has enough FTs to cover the amount they're offering
        assert!(cur_bal.ge(&casted_amount), "Not enough FTs in balance to cover offer: {:?}", amount);
        // if the buyer has enough FTs, subtract the amount from their balance
        self.ft_deposits.insert(&buyer_id, &(cur_bal.saturating_sub(casted_amount)));

        //process the purchase (which will remove the sale from the market and perform the transfer)
        self.process_purchase(
            contract_id,
            token_id,
            amount,
            buyer_id,
        );
    }

    //private function used when a sale is purchased. 
    //this will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties
    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        amount: U128,
        buyer_id: AccountId,
    ) -> Promise {
        //get the sale object by removing the sale
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        //initiate a cross contract call to the nft contract. This will transfer the token to the buyer
        ext_nft_contract::ext(nft_contract_id)
            // Attach 1 yoctoNEAR with static GAS equal to the GAS for nft transfer. Also attach an unused GAS weight of 1 by default.
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(GAS_FOR_NFT_TRANSFER)
            .nft_transfer(
                buyer_id.clone(), //purchaser (person to transfer the NFT to)
                token_id, //token ID to transfer
                Some(sale.approval_id), //market contract's approval ID in order to transfer the token on behalf of the owner
                Some("payout from market".to_string()) //memo (to include some context)
            )
        //after the transfer payout has been initiated, we resolve the promise by calling our own resolve_purchase function. 
        //resolve purchase will send the FTs to the owner of the sale if everything went well.
        .then(
            // No attached deposit with static GAS equal to the GAS for resolving the purchase. Also attach an unused GAS weight of 1 by default.
            Self::ext(env::current_account_id())
            .with_static_gas(GAS_FOR_RESOLVE_PURCHASE)
            .resolve_purchase(
                sale.owner_id, //the seller of the token
                buyer_id, //the buyer and price are passed in incase something goes wrong and we need to refund the buyer
                amount,
            )
        )
    }

    /*
        private method used to resolve the promise when calling nft_transfer_payout. This will
        transfer the tokens to the owner of the sale if the transfer was successful. If not, the buyer will be refunded.
        IMPORTANT - the seller MUST be registered on the FT contract before this function is called or else they will NOT
        receive their FTs
    */
    #[private]
    pub fn resolve_purchase(
        &mut self,
        seller_id: AccountId,
        buyer_id: AccountId,
        price: U128,
    ) -> U128 {
        let amount = NearToken::from_yoctonear(price.0);

        // Get the amount to revert the caller's balance with
        let transfer_amount = match env::promise_result(0) {
            // If the promise was successful, we'll transfer all the FTs
            PromiseResult::Successful(_) => {
                amount
            }
            // If the promise wasn't successful, we won't transfer any FTs and instead refund the buyer
            PromiseResult::Failed => NearToken::from_yoctonear(0),
        };

        // If the promise was successful, we'll transfer all the FTs
        if transfer_amount.gt(&NearToken::from_yoctonear(0)) {
            // Perform the cross contract call to transfer the FTs to the seller
            ext_ft_contract::ext(self.ft_id.clone())
                // Attach 1 yoctoNEAR with static GAS equal to the GAS for nft transfer. Also attach an unused GAS weight of 1 by default.
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_transfer(
                    seller_id, //seller to transfer the FTs to
                    U128(transfer_amount.as_yoctonear()), //amount to transfer
                    Some("Sale from marketplace".to_string()), //memo (to include some context)
                );
            return U128(transfer_amount.as_yoctonear());
        // If the promise was not successful, we won't transfer any FTs and instead refund the buyer
        } else {
            // Get the buyer's current balance and increment it
            let cur_bal = self.ft_deposits.get(&buyer_id).unwrap();
            self.ft_deposits.insert(&buyer_id, &(cur_bal.saturating_add(amount)));
            return U128(0);
        }
    }
}

//this is the cross contract call that we call on our own contract. 
/*
    private method used to resolve the promise when calling nft_transfer_payout. This will take the payout object and 
    check to see if it's authentic and there's no problems. If everything is fine, it will pay the accounts. If there's a problem,
    it will refund the buyer for the price. 
*/
#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
    ) -> Promise;
}
