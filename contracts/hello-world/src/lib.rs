// Specifies that this is a no-standard-library build, which is required for smart contracts.
#![no_std]

// Import the necessary components from the Soroban SDK.
use soroban_sdk::{
    contract,       // Macro to define a contract.
    contractimpl,   // Macro to implement the contract.
    contracttype,   // Macro to define a custom data type.
    log,            // For logging messages from the contract.
    Address,        // Soroban's data type for a user/contract address.
    Env,            // The contract's environment, gives access to storage, ledger, etc.
    String,         // Soroban's string type.
    Symbol,         // A short, efficient string type.
    symbol_short,   // Macro to create a Symbol.
};

// --- 1. DEFINE CUSTOM DATA TYPES ---

/**
 * @title TimeshareInfo
 * @dev This struct holds the specific details for a single timeshare token.
 * It's "Clone" so we can copy it, and "Debug/Eq/PartialEq" for testing/logging.
 */
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeshareInfo {
    pub hotel: String,  // e.g., "Grand Hotel"
    pub room: String,   // e.g., "Room 305"
    pub week: u32,      // e.g., 28 (for the 28th week of the year)
}

/**
 * @title DataKey
 * @dev We use this enum to create organized, unique keys for our contract's storage.
 * This is a common pattern in Soroban to avoid "key collisions".
 */
#[contracttype]
pub enum DataKey {
    Info(u64),  // Stores the TimeshareInfo for a specific token ID (u64)
    Owner(u64), // Stores the Address of the owner for a specific token ID (u64)
}

// --- 2. DEFINE CONSTANT STORAGE KEYS ---

// A key for storing the Address of the contract administrator (the "hotel admin").
const ADMIN: Symbol = symbol_short!("ADMIN");
// A key for storing a counter that generates unique token IDs.
const COUNTER: Symbol = symbol_short!("COUNTER");


// --- 3. DEFINE THE CONTRACT ---

/**
 * @title HotelTimeshareContract
 * @dev This is the main contract struct.
 */
#[contract]
pub struct HotelTimeshareContract;


// --- 4. IMPLEMENT THE CONTRACT LOGIC ---

/**
 * @title Implementation of HotelTimeshareContract
 * @dev This block contains all the public functions (endpoints) of our contract.
 * We will create 4 simple functions as requested:
 * 1. initialize: Sets up the contract (our "constructor").
 * 2. mint: Creates a new timeshare token (Admin only).
 * 3. transfer: Sends a token to a new owner (Owner only).
 * 4. get_info: Lets anyone see the details of a token.
 */
#[contractimpl]
impl HotelTimeshareContract {

    /**
     * @dev Initializes the contract by setting the administrator.
     * This function should only be called ONCE when the contract is deployed.
     * @param admin The address of the person/account who will be the "hotel admin".
     */
    pub fn initialize(env: Env, admin: Address) {
        // We check if the ADMIN key already exists in storage.
        // If it does, it means initialize() was already run, so we panic.
        if env.storage().instance().has(&ADMIN) {
            panic!("Contract already initialized");
        }

        // 1. Store the admin address in instance storage.
        env.storage().instance().set(&ADMIN, &admin);
        // 2. Initialize the token ID counter at 0.
        env.storage().instance().set(&COUNTER, &0u64);
    }

    /**
     * @dev Mints a new timeshare token and assigns it to an owner.
     * Only the contract ADMIN can call this function.
     * @param to The address that will receive the new token.
     * @param hotel The name of the hotel.
     * @param room The room number.
     * @param week The week of the year (1-52).
     * @return The unique token ID of the newly minted timeshare.
     */
    pub fn mint(env: Env, to: Address, hotel: String, room: String, week: u32) -> u64 {
        // 1. Load the admin address from storage.
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        // 2. This is the Soroban way to check authentication:
        // It requires that the 'admin' address has signed this transaction.
        admin.require_auth();

        // 3. Get the current token ID counter and increment it.
        let mut token_id: u64 = env.storage().instance().get(&COUNTER).unwrap();
        token_id += 1;

        // 4. Create the TimeshareInfo struct with the provided data.
        let info = TimeshareInfo { hotel, room, week };

        // 5. Store the new data using our DataKey enum.
        // Store the info (Hotel, Room, Week)
        env.storage().instance().set(&DataKey::Info(token_id), &info);
        // Store the owner
        env.storage().instance().set(&DataKey::Owner(token_id), &to);

        // 6. Save the new, incremented counter back to storage.
        env.storage().instance().set(&COUNTER, &token_id);

        // 7. Log a message (visible in the blockchain explorer).
        log!(&env, "Minted timeshare #{} for {}", token_id, to);

        // 8. Return the new token ID.
        token_id
    }

    /**
     * @dev Transfers a timeshare token from the current owner to a new owner.
     * Only the current owner of the token can authorize this.
     * @param from The current owner's address (who must sign).
     * @param to The new owner's address.
     * @param token_id The ID of the token to transfer.
     */
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u64) {
        // 1. Require authorization from the 'from' address.
        // This ensures the person calling this is who they say they are.
        from.require_auth();

        // 2. Get the key for this token's owner.
        let owner_key = DataKey::Owner(token_id);

        // 3. Check that the token exists.
        if !env.storage().instance().has(&owner_key) {
             panic!("Token does not exist");
        }

        // 4. Load the current owner from storage.
        let current_owner: Address = env.storage().instance().get(&owner_key).unwrap();

        // 5. Verify that the 'from' address is indeed the 'current_owner'.
        if current_owner != from {
            panic!("'from' address is not the owner");
        }

        // 6. If all checks pass, set the new owner.
        env.storage().instance().set(&owner_key, &to);

        // 7. Log the transfer.
        log!(&env, "Transferred token #{} from {} to {}", token_id, from, to);
    }

    /**
     * @dev A public, read-only function to get the details of a timeshare.
     * Anyone can call this function without authentication.
     * @param token_id The ID of the token to query.
     * @return The TimeshareInfo struct (hotel, room, week).
     */
    pub fn get_info(env: Env, token_id: u64) -> TimeshareInfo {
        let info_key = DataKey::Info(token_id);

        // .unwrap() will panic if the token_id doesn't exist,
        // which is the correct behavior (it can't return info that isn't there).
        env.storage().instance().get(&info_key).unwrap()
    }
}