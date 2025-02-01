use std::sync::{Arc, RwLock};

use litesvm::LiteSVM;
use mockchain_engine::rpc::{
    get_minimum_balance_for_rent_exemption::get_minimum_balance_for_rent_exemption,
    rpc::{Dependencies, RpcMethod, RpcRequest},
};
use serde_json::Value;

#[test]
fn test_no_params() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: None,
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = get_minimum_balance_for_rent_exemption(&req, &deps);
    assert_eq!(
        res,
        Err(serde_json::json!({
            "code": -32602,
            "message": "`params` should have at least 1 argument(s)"
        }))
    );
}

#[test]
fn test_minimum_balance() {
    let req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: RpcMethod::GetAccountInfo,
        params: Some(serde_json::json!([50])),
    };
    let deps = Dependencies {
        lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
    };
    let res = get_minimum_balance_for_rent_exemption(&req, &deps);
    assert_eq!(res, Ok(serde_json::json!(1238880)));
}
