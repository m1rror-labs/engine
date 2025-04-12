use bigdecimal::BigDecimal;
use dotenv::dotenv;
use mockchain_engine::storage::{accounts::DbAccount, blocks::DbBlock, cache::Cache};
use std::env;
use uuid::Uuid;

#[test]
fn test_set_account() {
    dotenv().ok();
    let database_url = env::var("CACHE_URL").expect("CACHE_URL must be set");
    let storage = Cache::new(&database_url);

    let pubkey = solana_sdk::pubkey::new_rand().to_string();
    let account = DbAccount {
        lamports: BigDecimal::from(100),
        data: vec![1, 2, 3],
        owner: solana_sdk::pubkey::new_rand().to_string(),
        executable: false,
        rent_epoch: BigDecimal::from(1234),
        id: Uuid::new_v4(),
        created_at: chrono::Utc::now().naive_utc(),
        address: pubkey.clone(),
        label: None,
        blockchain: Uuid::new_v4(),
    };

    let id = uuid::Uuid::new_v4();

    println!("ID: {}", id.to_string());

    storage.set_accounts(id, vec![account.clone()]).unwrap();

    let stored_account = storage.get_account(id, &pubkey.to_string()).unwrap();

    assert_eq!(pubkey.to_string(), stored_account.unwrap().address);

    storage.delete_blockchain(id).unwrap();
}

#[test]
fn test_set_blocks() {
    dotenv().ok();
    let database_url = env::var("CACHE_URL").expect("CACHE_URL must be set");
    let storage = Cache::new(&database_url);

    let blockhash = vec![1, 2, 3];
    let block = DbBlock {
        id: Uuid::new_v4(),
        created_at: chrono::Utc::now().naive_utc(),
        blockchain: Uuid::new_v4(),
        blockhash: blockhash.clone(),
        previous_blockhash: vec![4, 5, 6],
        parent_slot: BigDecimal::from(1234),
        block_height: BigDecimal::from(5678),
        slot: BigDecimal::from(91011),
    };

    let id = uuid::Uuid::new_v4();

    println!("ID: {}", id.to_string());

    storage.set_block(id, block).unwrap();

    let stored_block = storage.get_block(id, &blockhash).unwrap().unwrap();

    assert_eq!(blockhash, stored_block.blockhash);

    storage.delete_blockchain(id).unwrap();
}
