use std::vec;

use near_sdk::json_types::U128;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::{
    self, env, near_bindgen, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault, require, Promise,
};
use serde_json::json;

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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Log {
    pub id: String,
    pub timestamp: u64,
    pub action: String,
    pub actor: String,
    pub entity: String,
    pub extra: String,
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
    AuditLogs
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub overseer_id: AccountId,
    pub stores_by_account_id: Option<LookupMap<AccountId, UnorderedSet<StoreId>>>,
    pub owners_per_store_id: Option<LookupMap<StoreId, UnorderedSet<AccountId>>>,
    pub item_by_store_id: TreeMap<ItemId, StoreId>,
    pub metadata_by_storeanditem_ids: Option<UnorderedMap<StoreAndItemIds, ItemMetadata>>,
    pub audit_logs: UnorderedSet<Log>,
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
            audit_logs: UnorderedSet::new(StorageKey::AuditLogs.into_storage_key()),
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

    pub fn add_store_owners(&mut self, store_id: AccountId, new_owner_id: AccountId) {
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

            owner_ids.insert(&new_owner_id);
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

        let storeanditem_id = format!("{}{}{}", store_id, DELIMETER, item_id);
        let signer_id = env::predecessor_account_id();
        let deposit = env::attached_deposit();

        self.metadata_by_storeanditem_ids.as_mut().and_then({|by_id| {
            if let Some(metadata)= &mut by_id.get(&storeanditem_id) {
                require!(deposit >= metadata.price.0, "StoreHub: deposit is below price");
                require!(signer_id.ne(&metadata.owner), "StoreHub: can't buy owned item");

                Promise::new(metadata.owner.clone()).transfer(deposit);
                metadata.owner = signer_id.clone();
                
                by_id.insert(&storeanditem_id, &metadata);

                Some(())
            } else {
                None
            }
        }});

        let extra = json!({
            "paid": deposit,
            "previous_owner": store_id,
        });
        let tx_id = self.add_transaction("buy".to_string(), signer_id.to_string(), storeanditem_id, extra.to_string());

        json!({
            "message": "your purchase is ready",
            "transaction_id": tx_id,
        }).to_string()
    }

    pub fn add_transaction(&mut self, action: String, actor: String, entity: String, extra: String) -> String {
        let log_id = format!("{}{}{}", entity, DELIMETER, env::block_timestamp());
        let log = Log {
            id: log_id.clone(),
            timestamp: env::block_timestamp(),
            action,
            actor,
            entity,
            extra,
        };

        self.audit_logs.insert(&log);

        log_id
    }

    pub fn add_ft(&mut self, ft_account_id: AccountId) {
        require!(env::signer_account_id().eq(&self.overseer_id), "StoreHub: access denied");
        self.approved_ft_token_ids.insert(&ft_account_id);
    }
}

pub fn test_account() -> AccountId {
    AccountId::new_unchecked("testnet".to_string())
}