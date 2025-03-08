use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

pub fn get_largest_accounts<T: Storage + Clone + 'static>(
    id: Uuid,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let current_slot = match svm.get_latest_block(id) {
        Ok(blockhash) => blockhash,
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": e,
            }))
        }
    };

    match svm.get_largest_accounts(id) {
        Ok(accounts) => Ok(serde_json::json!({
            "context": {
                "slot": current_slot.block_height
              },
            "accounts": accounts.iter().map(|(account, balance)|{
                serde_json::json!({
                    "address": account.to_string(),
                    "balance": balance,
                })
            }).collect::<Vec<Value>>(),
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
