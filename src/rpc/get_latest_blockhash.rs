use serde_json::Value;
use solana_sdk::message::AddressLoader;
use uuid::Uuid;

use crate::{engine::SVM, storage::Storage};

use super::rpc::Dependencies;

pub fn get_latest_blockhash<T: Storage + AddressLoader>(
    id: Uuid,
    deps: &Dependencies<T>,
) -> Result<Value, Value> {
    let svm = deps.svm.read().unwrap();
    let blockhash = svm.latest_blockhash(id);
    Ok(serde_json::json!({
        "context": { "apiVersion": "2.0.15", "slot": 341197053 },
        "value": blockhash,
    }))
}
