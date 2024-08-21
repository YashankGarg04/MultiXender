use near_sdk::{env, log, near_bindgen, AccountId, Balance, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
//serialization is mostly used for storing data in the blockchain or we can say to 'bundle' the contract data
use near_sdk::collections::UnorderedMap;
use near_sdk::serde_json::from_str;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum ContractError {
    InsufficientBalance,
    JSONParsingError,
}
//this macro is added to wrap struct to make it NEAR compatible
#[near_bindgen]
//this macro is is used to wrap up the code into byte stream and hashing them
//hashing is also important so that multiple bytestream doesn't get mixed up
#[derive(BorshDeserialize, BorshSerialize)]//rust uses borsh to serialize and deserialize data
struct MultiSenderContract {
    sender_balances: UnorderedMap<AccountId, Balance>,//fields for the contract
}
//inheritance is used to inherit the fields from the parent class
//in rust it is done using impl keyword
impl Default for MultiSenderContract {//it is not mandatory to have this but it is good practice
    fn default() -> Self {
        Self {
            sender_balances: UnorderedMap::new(b"s".to_vec()),//UnorderedMap is a collection of key-value pairs here key is the account id and value is the balance
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Input {//different structs defined for sender and recipient
    recipients: Vec<RecipientAmount>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RecipientAmount {
    account_id: AccountId,
    amount: String,
}

#[near_bindgen]
impl MultiSenderContract {
    #[init]//this is the constructor for the contract while deploying contract it calls itself
    pub fn new() -> Self {
        Self {
            sender_balances: UnorderedMap::new(b"s".to_vec()),
        }
    }
    #[payable]//this is used to mark a function as payable , by default function is public and not payable also there can be private functions which can only be called by the owner of the contract
pub fn transfer(&mut self, input: String) -> Option<()> {
    // Parse the input as a JSON string
    let input: Input = match from_str(&input) {
        Ok(parsed) => parsed,
        Err(_) => {
            log!("JSON parsing error");
            return None;
        }
    };
    let sender_account_id = env::predecessor_account_id();//env is used to access the account specific environment variables
    let attached_amount = env::attached_deposit();
    // Ensure the transaction is signed by the account owner
    assert!(sender_account_id == env::signer_account_id(), "Transaction must be signed by the account owner");//assert is used to check the condition and if the condition is false it will panic at runtime

    log!("Sender: {}", sender_account_id);
    log!("Attached amount: {}", attached_amount);

    // Get sender's balance
    let sender_balance = attached_amount;

    // Iterate over recipients
    for recipient_amount in &input.recipients {
        let recipient = &recipient_amount.account_id;
        let amount_str = &recipient_amount.amount;
        // Parse the amount as a string and convert it to an integer
        let amount: u128 = match amount_str.parse() {
            Ok(parsed) => parsed,
            Err(_) => {
                log!("Invalid amount format");
                return Some(());
            }
        };

        log!("Recipient: {}", recipient);
        log!("Amount: {}", amount);

        // Check if sender has enough balance
        if sender_balance < amount {
            log!("Insufficient balance to transfer to {}", recipient);
            return None;
        }

        // Forward tokens to the recipient
        Promise::new(recipient.clone()).transfer(amount);

        // Update sender's balance
        self.sender_balances.insert(&sender_account_id, &(sender_balance - amount));

        // Update recipient's balance
        let recipient_balance = self.sender_balances.get(&recipient).unwrap_or(0);
        self.sender_balances.insert(&recipient, &(recipient_balance + amount));

        log!("Tokens sent from {} to {}", sender_account_id, recipient);
    }

    Some(()) // Return Some(()) to indicate success
}

    pub fn get_sender_balance(&self, account_id: AccountId) -> Balance { // this function is used to get the balance of the sender
        self.sender_balances.get(&account_id).unwrap_or(0)
    }
    //functions like this does not require a gas fee because they are not modifying the state of the contract
}
