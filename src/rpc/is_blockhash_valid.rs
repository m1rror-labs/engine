use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_hash, RpcRequest};

pub fn is_blockhash_valid<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let hash_str = match req
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
    let hash = match parse_hash(hash_str) {
        Ok(hash) => hash,
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": e,
            }));
        }
    };

    let (block, res) = match svm.is_blockhash_valid(id, &hash) {
        Ok((block, res)) => (block, res),
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": e,
            }));
        }
    };
    if res {
        Ok(serde_json::json!({
            "context": { "slot": block.block_height,"apiVersion":"2.1.13" },
            "value": true,
        }))
    } else {
        Ok(serde_json::json!({
            "context": { "slot": block.block_height,"apiVersion":"2.1.13" },
            "value": false,
        }))
    }
}
