use crate::*;

/// external contract calls

//initiate a cross contract call to the nft contract. This will transfer the token to the buyer
#[ext_contract(ext_nft_contract)]
trait ExtNftContract {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId, // purchaser (person to transfer the NFT to)
        token_id: TokenId, // token ID to transfer
        approval_id: Option<u64>, // market contract's approval ID in order to transfer the token on behalf of the owner
        memo: Option<String>, //memo (to include some context)
    );
}

//initiate a cross contract call to the nft contract. This will transfer the token to the buyer and return
//a payout object used for the market to distribute funds to the appropriate accounts.
#[ext_contract(ext_ft_contract)]
trait ExtFtContract {
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId, 
        amount: U128, 
        memo: Option<String>
    );
}