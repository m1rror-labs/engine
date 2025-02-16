use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub fn get_signatures_for_address<T: Storage + Clone + 'static>(
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
    let pubkey = parse_pubkey(pubkey_str)?;

    match svm.get_transactions_for_address(id, &pubkey, None) {
        Ok(transactions) => Ok(transactions
            .iter()
            .map(|tx| {
                serde_json::json!({
                    "err": null,
                    "memo": null,
                    "signature": tx.signature,
                    "slot": tx.slot,
                    "blockTime": null
                })
            })
            .collect::<Value>()),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
