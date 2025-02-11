// use base64::decode;
// use serde_json::Value;
// use solana_sdk::message::VersionedMessage;
// use uuid::Uuid;

// use crate::{
//     engine::{SvmEngine, SVM},
//     storage::Storage,
// };

// use super::rpc::RpcRequest;

// pub fn get_fee_for_message<T: Storage + Clone>(
//     id: Uuid,
//     req: &RpcRequest,
//     svm: &SvmEngine<T>,
// ) -> Result<Value, Value> {
//     let message_str = match req
//         .params
//         .as_ref()
//         .and_then(|params| params.get(0))
//         .and_then(|v| v.as_str())
//     {
//         Some(s) => s,
//         None => {
//             return Err(serde_json::json!({
//                 "code": -32602,
//                 "message": "`params` should have at least 1 argument(s)"
//             }));
//         }
//     };
//     let decoded_message = match decode(message_str) {
//         Ok(bytes) => bytes,
//         Err(e) => {
//             return Err(serde_json::json!({
//                 "code": -32602,
//                 "message": format!("Failed to decode base64: {}", e),
//             }));
//         }
//     };

//     let message: VersionedMessage = match serde_json::from_slice(&decoded_message) {
//         Ok(msg) => msg,
//         Err(e) => {
//             return Err(serde_json::json!({
//                 "code": -32602,
//                 "message": format!("Failed to deserialize message: {}", e),
//             }));
//         }
//     };

//     Ok(serde_json::json!({
//         "value": svm.get_fee_for_message( &santized_message),
//     }))
// }
