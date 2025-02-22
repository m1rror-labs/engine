use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::RpcRequest;

pub fn get_block_commitment<T: Storage + Clone + 'static>(
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

    match svm.get_block_confirmation_status(id, &block_height) {
        Ok(confirmation) => match confirmation {
            Some(_) => Ok(serde_json::json!({
                //TODO: I can mock this better
                "commitment": [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 10, 32
                  ],
                  "totalStake": 42
            })),
            None => Ok(serde_json::json!({
                "commitment": [],
                  "totalStake": 0
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
