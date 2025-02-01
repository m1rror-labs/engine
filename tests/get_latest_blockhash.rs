use std::sync::{Arc, RwLock};

use litesvm::LiteSVM;
use mockchain_engine::rpc::{get_latest_blockhash::get_latest_blockhash, rpc::Dependencies};

#[test]
fn test_get_latest_blockhash() {
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = get_latest_blockhash(&deps);
    assert_eq!(
        res,
        Ok(serde_json::json!({
                "context": {
                    "apiVersion": "2.0.15",
                    "slot": 341197053
                },
                "value": deps.lite_svm.read().unwrap().latest_blockhash().to_string()
        }))
    );
}
