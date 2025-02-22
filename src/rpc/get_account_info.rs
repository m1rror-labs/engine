use serde_json::Value;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{encode_account, parse_pubkey, RpcRequest};

pub fn get_account_info<T: Storage + Clone + 'static>(
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
    let config: Option<RpcAccountInfoConfig> = req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_object())
        .map(|map| serde_json::from_value(Value::Object(map.clone())))
        .transpose()
        .unwrap_or_default();
    let RpcAccountInfoConfig {
        encoding,
        data_slice,
        commitment,
        min_context_slot,
    } = config.unwrap_or_default();
    _ = commitment;
    _ = min_context_slot;

    let encoding = encoding.unwrap_or(UiAccountEncoding::Binary);
    let pubkey = parse_pubkey(pubkey_str)?;

    let slot = match svm.get_latest_block(id) {
        Ok(slot) => slot,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm.get_account(id, &pubkey) {
        Ok(account) => match account {
            Some(account) => {
                let account_data = match encode_account(&account, &pubkey, encoding, data_slice) {
                    Ok(data) => data,
                    Err(e) => {
                        return Err(serde_json::json!({
                            "code": -32002,
                            "message": e,
                        }))
                    }
                };
                Ok(serde_json::json!({
                    "context": { "slot": slot.block_height,"apiVersion":"2.1.13" },
                    "value": account_data,
                }))
            }
            None => Ok(serde_json::json!({
                "context": { "slot": slot.block_height,"apiVersion":"2.1.13" },
                "value": null,
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
