use serde_json::Value;

use super::rpc::{parse_hash, Dependencies, RpcRequest};

pub fn is_blockhash_valid(req: &RpcRequest, deps: &Dependencies) -> Result<Value, Value> {
    let hash_str = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_str())
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };
    let hash = parse_hash(hash_str)?;

    let lite_svm = deps.lite_svm.read().unwrap();
    let latest_hash = lite_svm.latest_blockhash();
    if hash == latest_hash {
        Ok(serde_json::json!({
            "context": { "slot": 341197053 },
            "value": true,
        }))
    } else {
        Ok(serde_json::json!({
            "context": { "slot": 341197053 },
            "value": false,
        }))
    }
}
