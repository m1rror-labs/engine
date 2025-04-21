use dotenv::dotenv;
use mockchain_engine::storage::{PgStorage, Storage};
use std::env;
use uuid::Uuid;

#[test]
fn test_read_team_from_api_key() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cache_url = env::var("CACHE_URL").expect("CACHE_URL must be set");
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let storage = PgStorage::new(&database_url, &cache_url, &rpc_url);

    let api_key = Uuid::parse_str("58f0e25e-583e-4280-aacb-9333c015a981").unwrap();

    let team_id = Uuid::parse_str("15b1eed5-6148-40ce-97dd-c0aaaa43bef0").unwrap();

    let team = storage.get_team_from_api_key(api_key).unwrap();

    assert_eq!(team.id, team_id);
}
