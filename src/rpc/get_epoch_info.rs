use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_epoch_info<T: Storage + Clone + 'static>(
    id: Uuid,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    match svm.get_latest_block(id) {
        Ok(block) => Ok(serde_json::json!({
            "absoluteSlot": block.block_height-10, //hardcoded
            "blockHeight": block.block_height-10,
            "epoch": 0,
            "slotIndex": block.block_height,
            "slotsInEpoch": 432000,
            "transactionCount": 151130291,
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
