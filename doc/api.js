/**
 * Initializes the contract with the given overseer_id.
 *
 * @function new
 * @param {string} overseer_id - The overseer account ID.
 * @returns {Contract} A new Contract instance.
 */
await contract.new(overseer_id);

/**
 * Adds a new store to the contract state. This is an action that changes the contract state.
 *
 * @function create_store
 * @param {string} store_id - The store ID to be added.
 * @returns {void}
 */
await contract.create_store(store_id);

/**
 * Adds a new owner to a store. This is an action that changes the contract state.
 *
 * @function add_store_owners
 * @param {string} store_id - The store ID.
 * @param {string} new_owner_id - The new owner's account ID.
 * @returns {void}
 */
await contract.add_store_owners(store_id, new_owner_id);

/**
 * Retrieves all stores owned by the predecessor account.
 *
 * @function get_stores_by_account_id
 * @returns {Array<string>} An array of account IDs of the stores owned by the predecessor account.
 */
await contract.get_stores_by_account_id();

/**
 * Retrieves all owners of a particular store.
 *
 * @function get_owners_by_store_id
 * @param {string} store_id - The store ID.
 * @returns {Array<string>} An array of account IDs of the owners of the specified store.
 */
await contract.get_owners_by_store_id(store_id);

/**
 * Retrieves an item from a particular store.
 *
 * @function get_item_by_store_id
 * @param {string} item_id - The item ID.
 * @returns {ItemMetadata} An object representing the item from the specified store.
 */
await contract.get_item_by_store_id(item_id);

/**
 * Adds a new item to a store. This is an action that changes the contract state.
 *
 * @function add_store_item
 * @param {string} item_id - The item ID.
 * @param {string} store_id - The store ID.
 * @param {string} item_name - The name of the item.
 * @param {U128} item_price - The price of the item.
 * @param {string} item_img_url - The image URL of the item.
 * @returns {void}
 */
await contract.add_store_item(item_id, store_id, item_name, item_price, item_img_url);

/**
 * Allows a user to buy an item from a store. This is an action that changes the contract state.
 *
 * @function buy
 * @param {string} item_id - The item ID.
 * @param {string} store_id - The store ID.
 * @returns {string} A JSON string containing a success message and the transaction ID.
 */
await contract.buy(item_id, store_id);

/**
 * Adds a log entry to the contract state. This is an action that changes the contract state.
 *
 * @function add_log
 * @param {string} action - The action that occurred.
 * @param {string} actor - The account ID that performed the action.
 * @param {string} entity - The entity on which the action was performed.
 * @param {string} extra - Any extra information about the log event.
 * @returns {string} The id of the log confirming the log was added.
 */
await contract.add_log(action, actor, entity, extra);


/**
 * Adds a fungible token (FT) to the contract state. This is an action that changes the contract state.
 *
 * @function add_ft
 * @param {string} ft_account_id - The account ID of the FT.
 * @returns {void}
 */
await contract.add_ft(ft_account_id);

/**
 * Retrieves a log entry from the contract state.
 *
 * @function get_log
 * @param {string} log_id - The log ID.
 * @returns {Log} The content of the specified log.
 */
await contract.get_log(log_id);

/**
 * Checks if a fungible token (FT) is approved.
 *
 * @function is_ft_approved
 * @param {string} ft_account_id - The account ID of the FT.
 * @returns {boolean} A boolean indicating whether the specified FT is approved.
 */
await contract.is_ft_approved(ft_account_id);