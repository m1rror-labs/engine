use serde_json::Value;

use super::rpc::Dependencies;

pub fn get_latest_blockhash(deps: &Dependencies) -> Result<Value, Value> {
    let lite_svm = deps.lite_svm.read().unwrap();
    let blockhash = lite_svm.latest_blockhash();
    Ok(serde_json::json!({
        "context": { "apiVersion": "2.0.15", "slot": 341197053 },
        "value": blockhash.to_string(),
    }))
}
