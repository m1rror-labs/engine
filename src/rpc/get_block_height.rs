use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_block_height<T: Storage + Clone + 'static>(
    id: Uuid,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    match svm.latest_blockhash(id) {
        Ok(block) => {
            println!("{:?}", block.block_height);
            Ok(serde_json::json!(block.block_height))
        }
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
