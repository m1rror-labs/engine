use serde_json::Value;
use solana_sdk::message::AddressLoader;
use uuid::Uuid;

use crate::{engine::SVM, storage::Storage};

use super::rpc::{parse_pubkey, Dependencies, RpcRequest};

pub fn get_balance<T: Storage + AddressLoader>(
    id: Uuid,
    req: &RpcRequest,
    deps: &Dependencies<T>,
) -> Result<Value, Value> {
    let pubkey_str = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_str())
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };
    let pubkey = parse_pubkey(pubkey_str)?;

    let svm = deps.svm.read().unwrap();
    match svm.get_balance(id, &pubkey) {
        Ok(balance) => match balance {
            Some(balance) => Ok(serde_json::json!({
                "context": { "slot": 341197053 },
                "value": balance,
            })),
            None => Ok(serde_json::json!({
                "context": { "slot": 341197053 },
                "value": 0,
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
