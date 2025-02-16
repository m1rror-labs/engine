use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::RpcRequest;

pub fn get_slot_leaders<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let num_leaders = match req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_u64())
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 2 argument(s)"
            }));
        }
    };

    match svm.get_identity(id) {
        Ok(pubkey) => {
            //Make an array of pubkey strings of length num_leaders
            let mut leaders = Vec::new();
            for _ in 0..num_leaders {
                leaders.push(pubkey.to_string());
            }

            Ok(serde_json::json!(leaders))
        }
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
