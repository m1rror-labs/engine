use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_latest_blockhash<T: Storage + Clone>(
    id: Uuid,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    match svm.latest_blockhash(id) {
        Ok(blockhash) => Ok(serde_json::json!({
            "context": {
                "slot": blockhash.block_height
              },
              "value": {
                "blockhash": blockhash.blockhash.to_string(),
                "lastValidBlockHeight": blockhash.block_height+100
              }
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32000,
            "message": e.to_string()
        })),
    }
}
