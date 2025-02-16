use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_identity<T: Storage + Clone + 'static>(id: Uuid, svm: &SvmEngine<T>) -> Result<Value, Value> {
    match svm.get_identity(id) {
        Ok(pubkey) => Ok(serde_json::json!({
            "value": pubkey.to_string(),
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
