use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_signature, RpcRequest};

pub fn get_signature_statuses<T: Storage + Clone>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let sig_str = match req
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
    let signature = parse_signature(sig_str)?;

    match svm.get_transaction(id, &signature) {
        Ok(transaction) => match transaction {
            Some((_, status)) => {
                let status_value = match status.err.clone() {
                    Some(err) => {
                        serde_json::json!({
                            "err": err,
                        })
                    }
                    None => {
                        serde_json::json!({
                            "status": {
                                "Ok": null
                            }
                        })
                    }
                };
                Ok(serde_json::json!({
                    "context": { "slot": 341197053 },
                    "value": [
                        {
                          "slot": status.slot,
                          "confirmations": null,
                          "err": status.err,
                          "status": status_value,
                          "confirmationStatus": status.confirmation_status,
                        },
                        null
                      ]
                }))
            }
            None => Ok(serde_json::json!({
                "context": { "slot": 341197053 },
                "value": null,
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
