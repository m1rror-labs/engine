use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

use super::rpc::{Dependencies, RpcRequest, RpcResponse};

pub fn get_account_info(req: RpcRequest, deps: &Dependencies) -> RpcResponse {
    let pubkey_str = match req.params.get(0).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return RpcResponse {
                jsonrpc: req.jsonrpc.clone(),
                id: req.id.clone(),
                result: None,
                error: Some(serde_json::json!({
                    "code": -32602,
                    "message": "`params` should have at least 1 argument(s)"
                })),
            };
        }
    };
    let pubkey = match Pubkey::from_str(pubkey_str) {
        Ok(pk) => pk,
        Err(_) => {
            return RpcResponse {
                jsonrpc: req.jsonrpc.clone(),
                id: req.id.clone(),
                result: None,
                error: Some(serde_json::json!({
                    "code": -32602,
                    "message": "Invalid params: unable to parse pubkey",
                })),
            };
        }
    };

    let lite_svm = deps.lite_svm.read().unwrap();

    if let Some(account) = lite_svm.get_account(&pubkey) {
        RpcResponse {
            jsonrpc: req.jsonrpc,
            id: req.id,
            result: Some(serde_json::json!({
                "context": { "apiVersion": "2.0.15", "slot": 341197053 },
                "value": {
                    "data": account.data,
                    "executable": account.executable,
                    "lamports": account.lamports,
                    "owner": account.owner.to_string(),
                    "rentEpoch": account.rent_epoch,
                },
            })),
            error: None,
        }
    } else {
        print!("Account not found");
        RpcResponse {
            jsonrpc: req.jsonrpc,
            id: req.id,
            result: Some(serde_json::json!({
                "context": { "apiVersion": "2.0.15", "slot": 341197053 },
                "value": null,
            })),
            error: None,
        }
    }
}
