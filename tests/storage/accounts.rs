use dotenv::dotenv;
use mockchain_engine::storage::{PgStorage, Storage};
use std::env;

#[test]
fn test_read_account() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cache_url = env::var("CACHE_URL").expect("CACHE_URL must be set");
    let storage = PgStorage::new(&database_url, &cache_url);

    let account = storage
        .get_account(uuid::Uuid::new_v4(), &solana_sdk::pubkey::new_rand())
        .unwrap();

    assert_eq!(account, None);
}

#[test]
fn test_set_account() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cache_url = env::var("CACHE_URL").expect("CACHE_URL must be set");
    let storage = PgStorage::new(&database_url, &cache_url);

    let account = solana_sdk::account::Account {
        lamports: 100,
        data: vec![1, 2, 3],
        owner: solana_sdk::pubkey::new_rand(),
        executable: false,
        rent_epoch: 18446744073708552000,
    };

    let id = uuid::Uuid::parse_str("110200f4-1a05-4a3f-b4f9-6bc38ff19cdf").unwrap();
    let address = solana_sdk::pubkey::new_rand();

    storage
        .set_account(id, &address, account.clone(), None)
        .unwrap();

    let stored_account = storage.get_account(id, &address).unwrap();

    assert_eq!(stored_account, Some(account));
}
