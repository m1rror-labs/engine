use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use litesvm::LiteSVM;
use mockchain_engine::rpc::{
    get_account_info::get_account_info,
    rpc::{Dependencies, RpcMethod, RpcRequest},
};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

#[test]
fn test_no_params() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!([])),
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = get_account_info(&req, &deps);
    assert_eq!(
        res,
        Err(serde_json::json!({
            "code": -32602,
            "message": "`params` should have at least 1 argument(s)"
        }))
    );
}

#[test]
fn test_bad_pubkey() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!(["Bad pubkey"])),
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = get_account_info(&req, &deps);
    assert_eq!(
        res,
        Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse pubkey"
        }))
    );
}

#[test]
fn test_uninitalized_account() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!([
            "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri"
        ])),
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = get_account_info(&req, &deps);
    assert_eq!(
        res,
        Ok(serde_json::json!({
                "context": {
                    "apiVersion": "2.0.15",
                    "slot": 341197053
                },
                "value": null
        }))
    );
}

#[test]
fn test_initialized_account() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!([
            "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri"
        ])),
    };
    let pubkey = Pubkey::from_str("83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri").unwrap();
    let mut svm = LiteSVM::new();
    svm.airdrop(&pubkey, 1000).unwrap();
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(svm)),
    };
    let res = get_account_info(&req, &deps);
    assert_eq!(
        res,
        Ok(serde_json::json!({
                "context": {
                    "apiVersion": "2.0.15",
                    "slot": 341197053
                },
                "value": {
                    "data": [],
                    "executable": false,
                    "lamports": 1000,
                    "owner": "11111111111111111111111111111111",
                    "rentEpoch": 0,
                    "space": 0
                }
        }))
    );
}
