use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub async fn get_balance<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
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

    let slot = match svm.get_latest_block(id) {
        Ok(slot) => slot,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm.get_balance(id, &pubkey).await {
        Ok(balance) => match balance {
            Some(balance) => Ok(serde_json::json!({
                "context": { "slot": slot.block_height,"apiVersion":"2.1.13" },
                "value": balance,
            })),
            None => Ok(serde_json::json!({
                "context": { "slot": slot.block_height,"apiVersion":"2.1.13" },
                "value": 0,
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
