// use std::sync::{Arc, RwLock};

// use litesvm::LiteSVM;
// use mockchain_engine::rpc::{
//     is_blockhash_valid::is_blockhash_valid,
//     rpc::{Dependencies, RpcMethod, RpcRequest},
// };
// use serde_json::Value;

// #[test]
// fn test_no_params() {
//     let req = RpcRequest {
//         jsonrpc: "2.0".to_string(),
//         id: Value::Number(1.into()),
//         method: RpcMethod::GetAccountInfo,
//         params: None,
//     };
//     let deps = Dependencies {
//         lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
//     };
//     let res = is_blockhash_valid(&req, &deps);
//     assert_eq!(
//         res,
//         Err(serde_json::json!({
//             "code": -32602,
//             "message": "`params` should have at least 1 argument(s)"
//         }))
//     );
// }

// #[test]
// fn test_bad_hash() {
//     let req = RpcRequest {
//         jsonrpc: "2.0".to_string(),
//         id: Value::Number(1.into()),
//         method: RpcMethod::GetAccountInfo,
//         params: Some(serde_json::json!(["Bad hash"])),
//     };
//     let deps = Dependencies {
//         lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
//     };
//     let res = is_blockhash_valid(&req, &deps);
//     assert_eq!(
//         res,
//         Err(serde_json::json!({
//             "code": -32602,
//             "message": "Invalid params: unable to parse hash"
//         }))
//     );
// }

// #[test]
// fn test_invalid() {
//     let req = RpcRequest {
//         jsonrpc: "2.0".to_string(),
//         id: Value::Number(1.into()),
//         method: RpcMethod::GetAccountInfo,
//         params: Some(serde_json::json!([
//             "J7rBdM6AecPDEZp8aPq5iPSNKVkU5Q76F3oAV4eW5wsW", {"commitment":"processed"}
//         ])),
//     };
//     let deps = Dependencies {
//         lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
//     };
//     let res = is_blockhash_valid(&req, &deps);
//     assert_eq!(
//         res,
//         Ok(serde_json::json!({
//                 "context": {
//                     "slot": 341197053
//                 },
//                 "value":false
//         }))
//     );
// }

// #[test]
// fn test_valid() {
//     let deps = Dependencies {
//         lite_svm: Arc::new(RwLock::new(LiteSVM::new())),
//     };
//     let hash = deps.lite_svm.read().unwrap().latest_blockhash();

//     let req = RpcRequest {
//         jsonrpc: "2.0".to_string(),
//         id: Value::Number(1.into()),
//         method: RpcMethod::GetAccountInfo,
//         params: Some(serde_json::json!([
//             hash.to_string(), {"commitment":"processed"}
//         ])),
//     };

//     let res = is_blockhash_valid(&req, &deps);
//     assert_eq!(
//         res,
//         Ok(serde_json::json!({
//                 "context": {
//                     "slot": 341197053
//                 },
//                 "value":true
//         }))
//     );
// }
