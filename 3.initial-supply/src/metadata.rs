use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen};

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub spec: String, // Should be ft-1.0.0 to indicate that a Fungible Token contract adheres to the current versions of this Metadata and the Fungible Token Core specs. This will allow consumers of the Fungible Token to know if they support the features of a given contract.
    pub name: String, // The human-readable name of the token.
    pub symbol: String, // The abbreviation, like wETH or AMPL.
    pub icon: Option<String>, // Icon of the fungible token.
    pub reference: Option<String>, // A link to a valid JSON file containing various keys offering supplementary details on the token 
    pub reference_hash: Option<Base64VecU8>, // The base64-encoded sha256 hash of the JSON file contained in the reference field. This is to guard against off-chain tampering.
    pub decimals: u8, // used in frontends to show the proper significant digits of a token. This concept is explained well in this OpenZeppelin post. https://docs.openzeppelin.com/contracts/3.x/erc20#a-note-on-decimals
}

pub trait FungibleTokenMetadataProvider {
    // View call for returning the contract metadata
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
