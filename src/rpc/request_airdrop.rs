use serde_json::Value;

use super::rpc::{parse_pubkey, Dependencies, RpcRequest};

pub fn request_airdrop(req: &RpcRequest, deps: &Dependencies) -> Result<Value, Value> {
    let pubkey_str = match req
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
    let pubkey = parse_pubkey(pubkey_str)?;

    let lamports = match req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_u64())
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 2 argument(s)"
            }));
        }
    };

    let mut lite_svm = deps.lite_svm.write().unwrap();
    match lite_svm.airdrop(&pubkey, lamports) {
        Ok(res) => Ok(serde_json::json!(res.signature.to_string())),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Failed to airdrop",
        })),
    }
}
