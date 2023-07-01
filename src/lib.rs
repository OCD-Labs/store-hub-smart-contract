use std::vec;

use near_sdk::json_types::U128;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    self, env, near_bindgen, require, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault,
    Promise,
};
use serde_json::json;

// every item metadata will have a unique ID which is `STOREID + DELIMITER + ITEM_ID`
static DELIMETER: &str = ".";

// Creating custom types to use within the contract. This makes things more readable.
pub type ItemId = String;
pub type StoreId = AccountId;
pub type StoreAndItemIds = String;

// Defines each item details
#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct ItemMetadata {
    pub name: String,
    pub price: U128,
    pub img_url: String,
    pub owner: AccountId,
}

// Defines action-driven event on each store
#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
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
    AuditLogs,
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
        let signer_id = env::signer_account_id();

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

    /// Create a new store for a particular account_id
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
                    .skip(0_usize)
                    .take(store_ids.len() as usize)
                    .collect(),
                None => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieve the owners of a store by its id
    pub fn get_owners_by_store_id(&self, store_id: AccountId) -> Vec<AccountId> {
        if let Some(owners_per_store_id) = &self.owners_per_store_id {
            match owners_per_store_id.get(&store_id) {
                Some(owner_ids) => owner_ids
                    .iter()
                    .skip(0_usize)
                    .take(owner_ids.len() as usize)
                    .collect(),
                None => vec![],
            }
        } else {
            vec![]
        }
    }

    /// Retrieve an existing item's metadata under a store.
    pub fn get_item_by_store_id(&self, item_id: ItemId) -> Option<ItemMetadata> {
        let store_id = self.item_by_store_id.get(&item_id)?;
        let store_and_item_id = format!("{}{}{}", store_id, DELIMETER, item_id);
        self.metadata_by_storeanditem_ids
            .as_ref()?
            .get(&store_and_item_id)
    }

    /// Add a new item and its metadata to an existing store
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
            owner: store_id.clone(),
        };

        let storeanditem_id = format!("{}{}{}", store_id, DELIMETER, item_id);

        self.metadata_by_storeanditem_ids
            .as_mut()
            .and_then(|by_id| by_id.insert(&storeanditem_id, &item_metadata));
    }

    /// Transfers assest across buyer and the store_id,
    /// creating a transaction log in the process
    #[payable]
    pub fn buy(&mut self, item_id: String, store_id: AccountId) -> String {
        // check both item and store exist, and be right places
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
        let signer_id = env::signer_account_id();
        let deposit = env::attached_deposit();

        // check deposit, ownership and update contract's state
        self.metadata_by_storeanditem_ids.as_mut().and_then({
            |by_id| {
                if let Some(metadata) = &mut by_id.get(&storeanditem_id) {
                    require!(
                        deposit >= metadata.price.0,
                        "StoreHub: deposit is below price"
                    );
                    require!(
                        signer_id.ne(&metadata.owner),
                        "StoreHub: can't buy owned item"
                    );

                    Promise::new(metadata.owner.clone()).transfer(deposit);
                    metadata.owner = signer_id.clone();

                    by_id.insert(&storeanditem_id, metadata);

                    Some(())
                } else {
                    None
                }
            }
        });

        // add new buy transaction log to state
        let extra = json!({
            "paid": deposit,
            "previous_owner": store_id,
        });
        let tx_id = self.add_log(
            "buy".to_string(),
            signer_id.to_string(),
            storeanditem_id,
            extra.to_string(),
        );

        json!({
            "message": "your purchase is ready",
            "transaction_id": tx_id,
        })
        .to_string()
    }

    /// Add a new audit log to the contract's state
    pub fn add_log(
        &mut self,
        action: String,
        actor: String,
        entity: String,
        extra: String,
    ) -> String {
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

    /// Add a new support payment means
    pub fn add_ft(&mut self, ft_account_id: AccountId) {
        require!(
            env::signer_account_id().eq(&self.overseer_id),
            "StoreHub: access denied"
        );
        self.approved_ft_token_ids.insert(&ft_account_id);
    }

    /// Retrieve a log by id
    pub fn get_log(&self, log_id: String) -> Log {
        match self.audit_logs.iter().find(|log| log.id == log_id) {
            Some(log) => log,
            None => {
                let msg = format!("No log found with id {}", log_id);
                env::panic_str(&msg)
            }
        }
    }

    /// Check if payment method exists
    pub fn is_ft_approved(&self, ft_account_id: AccountId) -> bool {
        self.approved_ft_token_ids.contains(&ft_account_id)
    }
}

pub fn test_account() -> AccountId {
    AccountId::new_unchecked("testnet".to_string())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    #[test]
    fn test_init_contract() {
        let context = VMContextBuilder::new();
        testing_env!(context.build());

        let contract = Contract::new(accounts(0));

        assert_eq!(contract.overseer_id, accounts(0).into());
    }

    #[test]
    fn test_create_store() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(1)).build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));

        assert_eq!(contract.get_stores_by_account_id(), vec![accounts(2)]);
    }

    #[test]
    fn test_add_store_owners() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(1)).build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));
        contract.add_store_owners(accounts(2), accounts(3));

        let owners = contract.get_owners_by_store_id(accounts(2));

        assert!(owners.contains(&accounts(3)));
    }

    #[test]
    fn test_add_store_item() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(1)).build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));

        let item_id = "item1".to_string();
        let item_name = "item_name".to_string();
        let item_price = U128(1000);
        let item_img_url = "http://image.url".to_string();

        contract.add_store_item(
            item_id.clone(),
            accounts(2),
            item_name.clone(),
            item_price,
            item_img_url.clone(),
        );

        let item = contract.get_item_by_store_id(item_id.clone());
        match item {
            Some(metadata) => {
                assert_eq!(metadata.name, item_name);
                assert_eq!(metadata.price, item_price);
                assert_eq!(metadata.img_url, item_img_url);
                assert_eq!(metadata.owner, accounts(2));
            }
            None => {
                assert!(false, "item doesn't exist");
            }
        }
    }

    #[test]
    #[should_panic(expected = "StoreHub: item doesn't exist")]
    fn test_buy_nonexistent_item() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .signer_account_id(accounts(1))
            .attached_deposit(1000)
            .build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));

        contract.buy("item1".to_string(), accounts(2));
    }

    #[test]
    #[should_panic(expected = "StoreHub: this item doesn't exist for this store")]
    fn test_buy_existent_wrong_store_item() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .signer_account_id(accounts(1))
            .attached_deposit(1000)
            .build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));

        let item_id = "item1".to_string();
        let item_name = "item_name".to_string();
        let item_price = U128(1000);
        let item_img_url = "http://image.url".to_string();

        contract.add_store_item(
            item_id.clone(),
            accounts(2),
            item_name.clone(),
            item_price,
            item_img_url.clone(),
        );

        contract.buy("item1".to_string(), accounts(3));
    }

    #[test]
    #[should_panic(expected = "StoreHub: can't buy owned item")]
    fn test_buy_owned_item() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(1)).build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));
        contract.add_store_item(
            "item1".to_string(),
            accounts(2),
            "item_name".to_string(),
            U128(1000),
            "http://image.url".to_string(),
        );

        testing_env!(context
            .signer_account_id(accounts(2))
            .attached_deposit(2000)
            .build());

        contract.buy("item1".to_string(), accounts(2));
    }

    #[test]
    #[should_panic(expected = "StoreHub: deposit is below price")]
    fn test_buy_with_insufficient_deposit() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .signer_account_id(accounts(3))
            .attached_deposit(500)
            .build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));
        contract.add_store_item(
            "item1".to_string(),
            accounts(2),
            "item_name".to_string(),
            U128(1000),
            "http://image.url".to_string(),
        );

        contract.buy("item1".to_string(), accounts(2));
    }

    #[test]
    fn test_buy_item() {
        let mut context = VMContextBuilder::new();
        testing_env!(context
            .signer_account_id(accounts(3))
            .attached_deposit(2000)
            .build());

        let mut contract = Contract::new(accounts(0));
        contract.create_store(accounts(2));
        contract.add_store_item(
            "item1".to_string(),
            accounts(2),
            "item_name".to_string(),
            U128(1000),
            "http://image.url".to_string(),
        );

        let response = contract.buy("item1".to_string(), accounts(2));
        let response: serde_json::Value = serde_json::from_str(&response).unwrap();

        assert_eq!(response["message"], "your purchase is ready");
        assert!(response["transaction_id"].is_string());
    }

    #[test]
    fn test_add_transaction() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(1)).build());

        let mut contract = Contract::new(accounts(0));
        let log_id = contract.add_log(
            "action".to_string(),
            "actor".to_string(),
            "entity".to_string(),
            "extra".to_string(),
        );

        let log = contract.get_log(log_id.clone());

        assert_eq!(log.id, log_id);
        assert_eq!(log.action, "action");
        assert_eq!(log.actor, "actor");
        assert_eq!(log.entity, "entity");
        assert_eq!(log.extra, "extra");
    }

    #[test]
    #[should_panic(expected = "StoreHub: access denied")]
    fn test_add_ft_denied() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(1)).build());

        let mut contract = Contract::new(accounts(0));
        contract.add_ft(accounts(2));
    }

    #[test]
    fn test_add_ft_approved() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.signer_account_id(accounts(0)).build());

        let mut contract = Contract::new(accounts(0));
        contract.add_ft(accounts(2));

        assert!(contract.is_ft_approved(accounts(2)));
    }
}