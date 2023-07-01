use std::vec;

use near_sdk::json_types::U128;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::{
    self, env, near_bindgen, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault,
};

//every transaction will have a unique ID which is `STOREID + DELIMITER + ITEM_ID`
static DELIMETER: &str = ".";

//Creating custom types to use within the contract. This makes things more readable.
pub type ItemId = String;
pub type StoreId = AccountId;
pub type StoreAndItemIds = String;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ItemMetadata {
    pub name: String,
    pub price: U128,
    pub img_url: String,
    pub owner: AccountId,
}

/// Helper structure for storage keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    StoresByAccountId,
    OwnersByStoreId,
    StoresByAccountIdInner { account_hash: Vec<u8> },
    OwnersByStoreIdInner { owner_hash: Vec<u8> },
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
    pub owners_per_store_id: Option<LookupMap<StoreId, UnorderedSet<AccountId>>>,
    pub item_by_store_id: TreeMap<ItemId, StoreId>,
    pub metadata_by_storeanditem_ids: Option<UnorderedMap<StoreAndItemIds, ItemMetadata>>,
    pub approved_ft_token_ids: UnorderedSet<AccountId>,
}

#[near_bindgen]
impl Contract {
    /// Initialize contract state
    #[init]
    pub fn new(overseer_id: AccountId) -> Self {
        let mut this = Self {
            overseer_id,
            stores_by_account_id: Some(LookupMap::new(
                StorageKey::StoresByAccountId.into_storage_key(),
            )),
            owners_per_store_id: Some(LookupMap::new(
                StorageKey::OwnersByStoreId.into_storage_key(),
            )),
            item_by_store_id: TreeMap::new(StorageKey::ItemByStoreId.into_storage_key()),
            metadata_by_storeanditem_ids: Some(UnorderedMap::new(
                StorageKey::TransactionsByStoreAndItemIds.into_storage_key(),
            )),
            approved_ft_token_ids: UnorderedSet::new(
                StorageKey::FungibleTokenIds.into_storage_key(),
            ),
        };

        this.approved_ft_token_ids.insert(&test_account());

        this
    }

    /// Add a new store
    pub fn create_store(&mut self, store_id: AccountId) {
        let signer_id = env::predecessor_account_id();

        if let Some(stores_by_account_id) = &mut self.stores_by_account_id {
            let mut store_ids = stores_by_account_id.get(&signer_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::StoresByAccountIdInner {
                    account_hash: env::sha256(signer_id.as_bytes()),
                })
            });

            store_ids.insert(&store_id);
            stores_by_account_id.insert(&signer_id, &store_ids);

            self.add_store_owners(store_id, signer_id)
        }
    }

    pub fn add_store_owners(&mut self, store_id: AccountId, owner_id: AccountId) {
        if let Some(owners_per_store_id) = &mut self.owners_per_store_id {
            let mut owner_ids = owners_per_store_id.get(&store_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::OwnersByStoreIdInner {
                    owner_hash: env::sha256(store_id.as_bytes()),
                })
            });

            let signer_id = env::predecessor_account_id();
            if signer_id != env::current_account_id() {
                if let Some(stores_by_account_id) = &self.stores_by_account_id {
                    let store_ids = stores_by_account_id.get(&signer_id).unwrap();
                    if !store_ids.contains(&store_id) {
                        env::panic_str("StoreHub: signer not store owner")
                    }
                };
            }

            owner_ids.insert(&owner_id);
            owners_per_store_id.insert(&store_id, &owner_ids);
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

    pub fn add_store_item(
        &mut self,
        item_id: String,
        store_id: AccountId,
        item_name: String,
        item_price: U128,
        item_img_url: String,
    ) {
        let signer_id = env::predecessor_account_id();
        match &self.owners_per_store_id {
            Some(owners_per_store_id) => {
                let owners_set = owners_per_store_id.get(&store_id).unwrap();
                if !owners_set.contains(&signer_id) {
                    env::panic_str("StoreHub: access denied")
                }
            }
            None => env::panic_str("StoreHub: internal contract error"),
        }

        self.item_by_store_id.insert(&item_id, &store_id);

        let item_metadata = ItemMetadata {
            name: item_name,
            price: item_price,
            img_url: item_img_url,
            owner: signer_id,
        };

        let storeanditem_id = format!("{}{}{}", store_id, DELIMETER, item_id);

        self.metadata_by_storeanditem_ids
            .as_mut()
            .and_then(|by_id| by_id.insert(&storeanditem_id, &item_metadata));
    }

    /// Transfers assest across buyer and the store_id,
    /// creating a transaction item in the process
    #[payable]
    pub fn buy(&mut self, item_id: String, store_id: AccountId) -> String {
        match self.item_by_store_id.get(&item_id) {
            Some(returned_store_id) => {
                if returned_store_id.ne(&store_id) {
                    env::panic_str("StoreHub: this item doesn't exist for this store")
                }
            }
            None => {
                env::panic_str("StoreHub: item doesn't exist");
            }
        }

        "".to_string()
    }
}

pub fn test_account() -> AccountId {
    AccountId::new_unchecked("testnet".to_string())
}