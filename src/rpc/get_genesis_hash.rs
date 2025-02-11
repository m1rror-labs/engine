use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_genesis_hash<T: Storage + Clone>(id: Uuid, svm: &SvmEngine<T>) -> Result<Value, Value> {
    match svm.get_genesis_hash(id) {
        Ok(hash) => Ok(serde_json::json!({
            "value": hash.to_string(),
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
