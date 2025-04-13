use super::{accounts::DbAccount, blocks::DbBlock, transactions::DbTransactionObject};
use base64::prelude::*;
use bigdecimal::ToPrimitive;
use redis::{Client, Commands};
use uuid::Uuid;

#[derive(Clone)]
pub struct Cache {
    client: Client,
}

// pub struct BlockchainCache {
//     // Accounts cache
//     pub accounts: HashMap<String, DbAccount>,

//     // Transactions cache
//     pub transactions: HashMap<String, TransactionMetadata>, // Transaction ID to Block ID mapping

//     // Blocks cache
//     pub blocks: HashMap<Uuid, Vec<DbBlock>>, // Block ID to Transaction IDs mapping
// }

impl Cache {
    pub fn new(url: &str) -> Self {
        Cache {
            client: Client::open(url).expect("Failed to create Redis client"),
        }
    }

    pub fn delete_blockchain(&self, blockchain: Uuid) -> Result<(), String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        let pattern = format!("blockchain:{}:*", blockchain);

        // Use SCAN to find all matching keys
        let mut cursor: u64 = 0;
        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100) // Fetch 100 keys at a time
                .query(&mut con)
                .map_err(|e| format!("Failed to scan keys: {}", e))?;

            // Delete the matching keys
            if !keys.is_empty() {
                let _: () = redis::cmd("DEL")
                    .arg(keys)
                    .query(&mut con)
                    .map_err(|e| format!("Failed to scan keys: {}", e))?;
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(())
    }

    pub fn set_accounts(&self, blockchain: Uuid, accounts: Vec<DbAccount>) -> Result<(), String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Prepare key-value pairs for MSET
        let mut key_value_pairs = Vec::new();
        for account in accounts {
            let key = format!(
                "blockchain:{}:account:{}",
                blockchain.to_string(),
                account.address,
            );
            let serialized_account = serde_json::to_string(&account)
                .map_err(|e| format!("Failed to serialize account: {}", e))?;
            key_value_pairs.push((key, serialized_account));
        }

        // Flatten the key-value pairs into a single vector for MSET
        let flattened: Vec<String> = key_value_pairs
            .into_iter()
            .flat_map(|(key, value)| vec![key, value])
            .collect();

        // Use MSET to set all accounts in one request
        let _: () = redis::cmd("MSET")
            .arg(flattened)
            .query(&mut con)
            .map_err(|e| format!("Failed to execute MSET: {}", e))?;

        Ok(())
    }

    pub fn get_account(
        &self,
        blockchain: Uuid,
        address: &str,
    ) -> Result<Option<DbAccount>, String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        let key = format!("blockchain:{}:account:{}", blockchain.to_string(), address);
        let raw_json: Option<String> = con
            .get(key)
            .map_err(|e| format!("Failed to scan keys: {}", e))?;
        let account = match raw_json {
            Some(json) => Some(
                serde_json::from_str::<DbAccount>(&json)
                    .map_err(|e| format!("Failed to deserialize: {}", e))?,
            ),
            None => None,
        };
        Ok(account)
    }

    pub fn get_accounts(
        &self,
        blockchain: Uuid,
        addresses: Vec<String>,
    ) -> Result<Vec<Option<DbAccount>>, String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Prepare the keys for MGET
        let keys: Vec<String> = addresses
            .iter()
            .map(|address| format!("blockchain:{}:account:{}", blockchain, address))
            .collect();

        // Execute MGET to fetch all keys in a single request
        let raw_jsons: Vec<Option<String>> = redis::cmd("MGET")
            .arg(keys)
            .query(&mut con)
            .map_err(|e| format!("Failed to execute MGET: {}", e))?;

        // Deserialize the results into DbAccount objects
        let accounts: Vec<Option<DbAccount>> = raw_jsons
            .into_iter()
            .map(|raw_json| {
                raw_json
                    .map(|json| {
                        serde_json::from_str::<DbAccount>(&json)
                            .map_err(|e| format!("Failed to deserialize: {}", e))
                    })
                    .transpose()
            })
            .collect::<Result<Vec<Option<DbAccount>>, String>>()?;

        Ok(accounts)
    }

    pub fn set_block(&self, blockchain: Uuid, block: DbBlock) -> Result<(), String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Define the sorted set key
        let sorted_set_key = format!("blockchain:{}:block", blockchain.to_string());

        // Define the individual block key
        let block_key = format!(
            "blockchain:{}:block:{}",
            blockchain.to_string(),
            BASE64_STANDARD.encode(&block.blockhash)
        );

        // Serialize the block to JSON
        let serialized_block = serde_json::to_string(&block)
            .map_err(|e| format!("Failed to serialize block: {}", e))?;

        // Use the block's height or timestamp as the score
        let score = block.block_height.to_u64().unwrap() as f64; // Or use block.timestamp as f64

        // Add the block to the sorted set
        let _: () = redis::cmd("ZADD")
            .arg(&sorted_set_key)
            .arg(score)
            .arg(serialized_block.clone())
            .query(&mut con)
            .map_err(|e| format!("Failed to add block to sorted set: {}", e))?;

        // Store the block individually
        let _: () = redis::cmd("SET")
            .arg(&block_key)
            .arg(serialized_block)
            .query(&mut con)
            .map_err(|e| format!("Failed to store individual block: {}", e))?;

        Ok(())
    }

    pub fn get_block(&self, blockchain: Uuid, blockhash: &[u8]) -> Result<Option<DbBlock>, String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        let key = format!(
            "blockchain:{}:block:{}",
            blockchain.to_string(),
            BASE64_STANDARD.encode(blockhash)
        );
        let raw_json: Option<String> = con
            .get(key)
            .map_err(|e| format!("Failed to scan keys: {}", e))?;
        let block = match raw_json {
            Some(json) => Some(
                serde_json::from_str::<DbBlock>(&json)
                    .map_err(|e| format!("Failed to deserialize: {}", e))?,
            ),
            None => None,
        };
        Ok(block)
    }

    pub fn get_latest_block(&self, blockchain: Uuid) -> Result<DbBlock, String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Define the Redis key for the sorted set
        let key = format!("blockchain:{}:block", blockchain);

        // Fetch the most recent block (highest score) using ZREVRANGE
        let raw_json: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(&key)
            .arg(0) // Start index
            .arg(0) // End index (only the most recent block)
            .query(&mut con)
            .map_err(|e| format!("Failed to fetch latest block: {}", e))?;

        // Check if the vector contains any elements and deserialize the first one
        if let Some(json) = raw_json.into_iter().next() {
            let block = serde_json::from_str::<DbBlock>(&json)
                .map_err(|e| format!("Failed to deserialize block: {}", e))?;
            Ok(block)
        } else {
            Err("No blocks found".to_string())
        }
    }

    pub fn set_transaction(
        &self,
        blockchain: Uuid,
        transaction: DbTransactionObject,
    ) -> Result<(), String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        let key = format!(
            "blockchain:{}:transaction:{}",
            blockchain.to_string(),
            transaction.transaction.signature,
        );
        let serialized_transaction = serde_json::to_string(&transaction)
            .map_err(|e| format!("Failed to deserialize: {}", e))?;
        let _: () = con
            .set(key, serialized_transaction)
            .map_err(|e| format!("Failed to scan keys: {}", e))?;
        Ok(())
    }

    pub fn get_transaction(
        &self,
        blockchain: Uuid,
        signature: &str,
    ) -> Result<Option<DbTransactionObject>, String> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        let key = format!(
            "blockchain:{}:transaction:{}",
            blockchain.to_string(),
            signature
        );
        let raw_json: Option<String> = con
            .get(key)
            .map_err(|e| format!("Failed to scan keys: {}", e))?;
        let transaction = match raw_json {
            Some(json) => Some(
                serde_json::from_str::<DbTransactionObject>(&json)
                    .map_err(|e| format!("Failed to deserialize: {}", e))?,
            ),
            None => None,
        };
        Ok(transaction)
    }
}
