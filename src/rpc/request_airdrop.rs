use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub async fn request_airdrop<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
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
    let pubkey = match parse_pubkey(pubkey_str) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": e,
            }));
        }
    };

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

    match svm.airdrop(id, &pubkey, lamports).await {
        Ok(sig) => Ok(serde_json::json!(sig.to_string())),
        Err(e) => Err(serde_json::json!({
            "code": -32000,
            "message": e.to_string()
        })),
    }
}
