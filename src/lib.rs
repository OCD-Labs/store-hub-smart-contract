use near_sdk::collections::{LookupMap, UnorderedSet, TreeMap, UnorderedMap};
use near_sdk::{self, near_bindgen, PanicOnDefault, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

//every transaction will have a unique ID which is `STOREID + DELIMITER + ITEM_ID`
static DELIMETER: &str = ".";

//Creating custom types to use within the contract. This makes things more readable.
pub type ItemId = String;
pub type StoreId = AccountId;
pub type StoreIdAndItemId = String;

pub struct ItemMetadata {
    pub name: String,
    pub price: String,
    pub img_url: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub overseer_id: AccountId,
    pub stores_per_account_id: Option<LookupMap<AccountId, UnorderedSet<StoreId>>>,
    pub item_by_store_id: TreeMap<ItemId, StoreId>,
    pub transactions: Option<UnorderedMap<StoreIdAndItemId, ItemMetadata>>
}

impl Contract {
    
}

