use serde_json::Value;

use super::rpc::{parse_pubkey, Dependencies, RpcRequest};

pub fn get_balance(req: &RpcRequest, deps: &Dependencies) -> Result<Value, Value> {
    let pubkey_str = match req.params.get(0).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };
    let pubkey = parse_pubkey(pubkey_str)?;

    let lite_svm = deps.lite_svm.read().unwrap();

    if let Some(balance) = lite_svm.get_balance(&pubkey) {
        Ok(serde_json::json!({
            "context": { "apiVersion": "2.0.15", "slot": 341197053 },
            "value": balance,
        }))
    } else {
        Ok(serde_json::json!({
            "context": { "apiVersion": "2.0.15", "slot": 341197053 },
            "value": 0,
        }))
    }
}
