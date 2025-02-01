use serde_json::Value;

use super::rpc::{parse_pubkey, Dependencies, RpcRequest};

pub fn get_account_info(req: &RpcRequest, deps: &Dependencies) -> Result<Value, Value> {
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

    if let Some(account) = lite_svm.get_account(&pubkey) {
        Ok(serde_json::json!({
            "context": { "apiVersion": "2.0.15", "slot": 341197053 },
            "value": {
                "data": account.data,
                "executable": account.executable,
                "lamports": account.lamports,
                "owner": account.owner.to_string(),
                "rentEpoch": account.rent_epoch,
                "space": account.data.len(),
            },
        }))
    } else {
        Ok(serde_json::json!({
            "context": { "apiVersion": "2.0.15", "slot": 341197053 },
            "value": null,
        }))
    }
}
