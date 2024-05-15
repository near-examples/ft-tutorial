use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, NearToken, BorshStorageKey, PanicOnDefault, StorageUsage, NearSchema};

pub mod ft_core;
pub mod events;
pub mod metadata;
pub mod storage;
pub mod internal;

use crate::metadata::*;
use crate::events::*;

/// The image URL for the default icon
const DATA_IMAGE_SVG_GT_ICON: &str = "data:image/jpeg;base64,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";

/// The specific version of the standard we're using
pub const FT_METADATA_SPEC: &str = "ft-1.0.0";

pub const ZERO_TOKEN: NearToken = NearToken::from_yoctonear(0);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    /// Keep track of each account's balances
    pub accounts: LookupMap<AccountId, NearToken>,

    /// Total supply of all tokens.
    pub total_supply: NearToken,

    /// The bytes for the largest possible account ID that can be registered on the contract 
    pub bytes_for_longest_account_id: StorageUsage,

    /// Metadata for the contract itself
    pub metadata: LazyOption<FungibleTokenMetadata>,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Accounts,
    Metadata
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        // Calls the other function "new: with some default metadata and the owner_id & total supply passed in 
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Team Token FT Tutorial".to_string(),
                symbol: "gtNEAR".to_string(),
                icon: Some(DATA_IMAGE_SVG_GT_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        let casted_total_supply = NearToken::from_yoctonear(total_supply.0);
        // Create a variable of type Self with all the fields initialized. 
        let mut this = Self {
            // Set the total supply
            total_supply: casted_total_supply,
            // Set the bytes for the longest account ID to 0 temporarily until it's calculated later
            bytes_for_longest_account_id: 0,
            // Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            accounts: LookupMap::new(StorageKey::Accounts),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&metadata),
            ),
        };

        // Measure the bytes for the longest account ID and store it in the contract.
        this.measure_bytes_for_longest_account_id();

        // Register the owner's account and set their balance to the total supply.
        this.internal_register_account(&owner_id);
        this.internal_deposit(&owner_id, casted_total_supply);
        
        // Emit an event showing that the FTs were minted
        FtMint {
            owner_id: &owner_id,
            amount: &casted_total_supply,
            memo: Some("Initial token supply is minted"),
        }
        .emit();

        // Return the Contract object
        this
    }
}