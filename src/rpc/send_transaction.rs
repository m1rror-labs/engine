use chrono::Utc;
use serde_json::Value;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::{bpf_loader, bpf_loader_upgradeable, transaction::VersionedTransaction};
use solana_transaction_status_client_types::UiTransactionEncoding;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{decode_and_deserialize, RpcRequest};

pub fn send_transaction<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    println!(
        "Current date/time is sending tx: {}",
        Utc::now().to_rfc3339()
    );
    let tx_data = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| Some(v))
    {
        Some(s) => s.as_str().ok_or_else(|| {
            serde_json::json!({
                "code": -32602,
                "message": "`params[0]` should be a string"
            })
        })?,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };
    let config: Option<RpcSendTransactionConfig> = req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_object())
        .map(|map| serde_json::from_value(Value::Object(map.clone())))
        .transpose()
        .unwrap_or_default();

    let RpcSendTransactionConfig {
        skip_preflight,
        preflight_commitment,
        encoding,
        max_retries,
        min_context_slot,
    } = config.unwrap_or_default();
    _ = preflight_commitment;
    _ = max_retries;
    _ = min_context_slot;
    let tx_encoding = encoding.unwrap_or(UiTransactionEncoding::Base58);
    let binary_encoding = tx_encoding.into_binary_encoding().ok_or_else(|| {
        format!("unsupported encoding: {tx_encoding}. Supported encodings: base58, base64")
    })?;
    let (_, unsanitized_tx) =
        match decode_and_deserialize::<VersionedTransaction>(tx_data.to_owned(), binary_encoding) {
            Ok(tx) => tx,
            Err(e) => {
                return Err(serde_json::json!({
                    "code": -32602,
                    "message": e,
                }));
            }
        };

    let _ = match unsanitized_tx.sanitize() {
        Ok(tx) => tx,
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": e.to_string(),
            }));
        }
    };

    if unsanitized_tx
        .message
        .instructions()
        .iter()
        .map(|ix| ix.program_id(unsanitized_tx.message.static_account_keys()))
        .any(|program_id| {
            program_id.to_owned() == bpf_loader::id()
                || program_id.to_owned() == bpf_loader_upgradeable::id()
        })
    {
        return Err(serde_json::json!({
            "code": -32602,
            "message": "Uploading programs is not allowed, please use the UI at https://app.mirror.ad to upload programs for now. If running anchor test, run anchor test --skip-deploy",
        }));
    }

    if !skip_preflight {
        match svm.simulate_transaction(id, unsanitized_tx.clone()) {
            Ok(_) => {}
            Err(e) => {
                return Err(serde_json::json!({
                    "code": -32602,
                    "message": e,
                }));
            }
        }
    }
    println!(
        "Current date/time about to send tx: {}",
        Utc::now().to_rfc3339()
    );
    match svm.send_transaction(id, unsanitized_tx) {
        Ok(res) => Ok(serde_json::json!(res)),
        Err(e) => Err(serde_json::json!({
            "code": -32602,
            "message": e,
        })),
    }
}
