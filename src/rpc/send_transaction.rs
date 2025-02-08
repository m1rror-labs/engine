use serde_json::Value;
use solana_sdk::message::AddressLoader;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_tx, RpcRequest};

pub fn send_transaction<T: Storage + AddressLoader>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let tx = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| Some(v))
    {
        Some(s) => match parse_tx(s.clone()) {
            Ok(tx) => tx,
            Err(_) => {
                return Err(serde_json::json!({
                    "code": -32602,
                    "message": "Invalid params: unable to parse tx"
                }));
            }
        },
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };

    match svm.send_transaction(id, tx) {
        Ok(res) => Ok(serde_json::json!(res)),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Failed to send tx",
        })),
    }
}
