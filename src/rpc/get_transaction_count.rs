use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_hash, RpcRequest};

pub fn get_transaction_count<T: Storage + Clone>(
    id: Uuid,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    match svm.get_transaction_count(id) {
        Ok(count) => Ok(serde_json::json!(count)),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
