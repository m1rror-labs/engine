use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use litesvm::LiteSVM;
use mockchain_engine::rpc::{
    request_airdrop::request_airdrop,
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
    let res = request_airdrop(&req, &deps);
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
    let res = request_airdrop(&req, &deps);
    assert_eq!(
        res,
        Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse pubkey"
        }))
    );
}

#[test]
fn test_no_lmaports() {
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
    let res = request_airdrop(&req, &deps);
    assert_eq!(
        res,
        Err(serde_json::json!({
            "code": -32602,
            "message": "`params` should have at least 2 argument(s)"
        }))
    );
}

#[test]
fn test_bad_lmaports() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!([
            "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri",
            ""
        ])),
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = request_airdrop(&req, &deps);
    assert_eq!(
        res,
        Err(serde_json::json!({
            "code": -32602,
            "message": "`params` should have at least 2 argument(s)" //TODO: This should be "Invalid params: unable to parse lmaports"
        }))
    );
}

#[test]
fn test_aidrop_success() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!([
            "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri",
            1000
        ])),
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    request_airdrop(&req, &deps).unwrap();
    let svm = deps.lite_svm.read().unwrap();
    let pubkey = Pubkey::from_str("83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri").unwrap();
    let account = svm.get_account(&pubkey).unwrap();
    assert_eq!(account.lamports, 1000);
}

//TODO: Test aidrop failure
