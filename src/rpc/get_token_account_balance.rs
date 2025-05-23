use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub async fn get_token_account_balance<T: Storage + Clone + 'static>(
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
    let pubkey = match parse_pubkey(pubkey_str) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": e,
            }));
        }
    };

    let blockchain = match svm.storage.get_blockchain(id) {
        Ok(blockchain) => blockchain,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm
        .get_token_account_balance(id, &pubkey, blockchain.jit)
        .await
    {
        Ok(amount) => match amount {
            Some(amount) => Ok(serde_json::json!({
                "context": { "slot": 341197053,"apiVersion":"2.1.13" },
                "value":  amount,
            })),
            None => Err(serde_json::json!({
                "code": -32602,
                "message": "Invalid param: could not find account"
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
