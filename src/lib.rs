use near_sdk::collections::{LookupMap, TreeMap, UnorderedSet};
use near_sdk::{self, near_bindgen, AccountId, PanicOnDefault};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

pub type ItemId = String;
pub type StoreId = String;

pub struct ItemMetadata {
    pub name: String,
    pub price: String,
    pub img_url: String,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]

pub struct Contract {
    pub overseer_id: AccountId,
    pub stores_per_account_id: Option<LookupMap<AccountId, UnorderedSet<StoreId>>>,
    pub item_by_store_id: TreeMap<ItemId, StoreId>,
    pub item_metadata_by_id: Option<LookupMap<ItemId, ItemMetadata>>,
}

impl Contract {}
