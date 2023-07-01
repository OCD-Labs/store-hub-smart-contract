use std::vec;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::{
    self, env, near_bindgen, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault,
};

//every transaction will have a unique ID which is `STOREID + DELIMITER + ITEM_ID`
// static DELIMETER: &str = ".";

//Creating custom types to use within the contract. This makes things more readable.
pub type ItemId = String;
pub type StoreId = AccountId;
pub type StoreAndItemIds = String;

pub struct ItemMetadata {
    pub name: String,
    pub price: String,
    pub img_url: String,
}

/// Helper structure for storage keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    StoresByAccountId,
    StoressByAccountId { account_hash: Vec<u8> },
    ItemByStoreId,
    TransactionsByStoreAndItemIds,
    FungibleTokenIds,
    StoresPerOwnerInner,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub overseer_id: AccountId,
    pub stores_by_account_id: Option<LookupMap<AccountId, UnorderedSet<StoreId>>>,
    pub item_by_store_id: TreeMap<ItemId, StoreId>,
    pub transactions_by_storeanditem_ids: Option<UnorderedMap<StoreAndItemIds, ItemMetadata>>,
    pub approved_ft_token_ids: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl Contract {
    /// Initialize contract state
    #[init]
    pub fn new(overseer_id: AccountId) -> Self {
        Self {
            overseer_id,
            stores_by_account_id: Some(LookupMap::new(
                StorageKey::StoresByAccountId.into_storage_key(),
            )),
            item_by_store_id: TreeMap::new(StorageKey::ItemByStoreId.into_storage_key()),
            transactions_by_storeanditem_ids: Some(UnorderedMap::new(
                StorageKey::TransactionsByStoreAndItemIds.into_storage_key(),
            )),
            approved_ft_token_ids: UnorderedSet::new(
                StorageKey::FungibleTokenIds.into_storage_key(),
            ),
        }
    }

    /// Add a new store
    pub fn create_store(&mut self, store_id: AccountId) {
        let signer_id = env::predecessor_account_id();

        if let Some(stores_by_account_id) = &mut self.stores_by_account_id {
            let mut store_ids = stores_by_account_id.get(&signer_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::StoressByAccountId {
                    account_hash: env::sha256(signer_id.as_bytes()),
                })
            });

            store_ids.insert(&store_id);
            stores_by_account_id.insert(&signer_id, &store_ids);
        }
    }

    /// Retrieve a stores owned by signer_id
    pub fn get_stores_by_account_id(&self) -> Vec<AccountId> {
        let signer_id = env::predecessor_account_id();

        if let Some(stores_by_account_id) = &self.stores_by_account_id {
            match stores_by_account_id.get(&signer_id) {
                Some(store_ids) => store_ids
                    .iter()
                    .skip(0 as usize)
                    .take(store_ids.len() as usize)
                    .map(|store_id| store_id)
                    .collect(),
                None => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Transfers assest across buyer and the store_id, 
    /// creating a transaction item in the process
    pub fn buy(&mut self) -> String {
        "".to_string()
    }
}