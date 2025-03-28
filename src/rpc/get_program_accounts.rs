use serde_json::Value;
use solana_account_decoder_client_types::UiAccountEncoding;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{encode_account, parse_pubkey, RpcRequest};

pub fn get_program_accounts<T: Storage + Clone + 'static>(
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

    let slot = match svm.get_latest_block(id) {
        Ok(slot) => slot,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm.get_program_accounts(id, &pubkey) {
        Ok(accounts) => Ok(serde_json::json!({
            "context": { "slot": slot.block_height,"apiVersion":"2.1.13" },

            "accounts": accounts.iter().map(|(pubkey, account)| {
            match encode_account(account, &pubkey, UiAccountEncoding::Base64,None, None) {
                    Ok(encoded_account) => serde_json::json!(encoded_account),
                    Err(e) => {
                        return serde_json::json!({
                            "error": {
                                "code": -32002,
                                "message": e,
                            },
                        });
                    }
                }
            }).collect::<Vec<Value>>(),
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
