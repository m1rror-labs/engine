use serde_json::Value;

use super::rpc::{parse_tx, Dependencies, RpcRequest};

pub fn send_transaction(req: &RpcRequest, deps: &Dependencies) -> Result<Value, Value> {
    let tx = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| Some(v))
    {
        Some(s) => match parse_tx(s.clone()) {
            Ok(tx) => tx,
            Err(_) => {
                return Err(serde_json::json!({
                    "code": -32602,
                    "message": "Invalid params: unable to parse tx"
                }));
            }
        },
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };

    let mut lite_svm = deps.lite_svm.write().unwrap();
    match lite_svm.send_transaction(tx) {
        Ok(res) => Ok(serde_json::json!(res.signature.to_string())),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Failed to send tx",
        })),
    }
}
