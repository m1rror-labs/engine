use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::RpcRequest;

pub fn get_block_time<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let block_height = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_u64())
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };

    match svm.get_block(id, &block_height) {
        Ok(block) => match block {
            Some(block) => Ok(serde_json::json!({
                "context": { "slot": block_height,"apiVersion":"2.1.13" },
                "value": {
                    "blockTime": block.block_time,
                }
            })),
            None => Err(serde_json::json!({
                "code": -32002,
                "message": "Block not found",
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
