use serde_json::Value;
use solana_sdk::message::AddressLoader;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_latest_blockhash<T: Storage + AddressLoader>(
    id: Uuid,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let blockhash = svm.latest_blockhash(id);
    Ok(serde_json::json!({
        "context": { "apiVersion": "2.0.15", "slot": 341197053 },
        "value": blockhash,
    }))
}
