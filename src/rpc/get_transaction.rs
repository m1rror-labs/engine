use jsonrpc_core::Result as JsonResult;
use serde_json::Value;
use solana_rpc_client_api::{config::RpcTransactionConfig, custom_error::RpcCustomError};
use solana_sdk::{
    instruction::AccountMeta,
    message::{v0::LoadedAddresses, VersionedMessage},
    transaction::{TransactionError, VersionedTransaction},
};
use solana_transaction_status::{
    ConfirmedTransactionWithStatusMeta, EncodedConfirmedTransactionWithStatusMeta,
    InnerInstructions, TransactionStatusMeta, TransactionTokenBalance, TransactionWithStatusMeta,
    UiTransactionEncoding, VersionedTransactionWithStatusMeta,
};
use solana_transaction_status_client_types::InnerInstruction;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_signature, RpcRequest};

pub fn get_transaction<T: Storage + Clone + 'static>(
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
    let signature = match parse_signature(sig_str) {
        Ok(signature) => signature,
        Err(e) => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": e,
            }));
        }
    };
    let config: Option<RpcTransactionConfig> = req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_object())
        .map(|map| serde_json::from_value(Value::Object(map.clone())))
        .transpose()
        .unwrap_or_default();
    let RpcTransactionConfig {
        encoding,
        commitment,
        max_supported_transaction_version,
    } = config.unwrap_or_default();
    _ = commitment;
    _ = max_supported_transaction_version;
    let encoding = encoding.unwrap_or(UiTransactionEncoding::Base64);

    let slot = match svm.get_latest_block(id) {
        Ok(slot) => slot,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    let encode_transaction =
    |confirmed_tx_with_meta: ConfirmedTransactionWithStatusMeta| -> JsonResult<EncodedConfirmedTransactionWithStatusMeta> {
        Ok(confirmed_tx_with_meta.encode(encoding, max_supported_transaction_version).map_err(RpcCustomError::from)?)
    };

    match svm.get_transaction(id, &signature) {
        Ok(transaction) => {
            match transaction {
                Some((transaction, tx_meta, status)) => {
                    let versioned_message = VersionedMessage::Legacy(transaction.message().clone());
                    let versioned_transaction = VersionedTransaction {
                        message: versioned_message,
                        signatures: transaction.signatures.clone(),
                    };
                    let status = match tx_meta.clone().err {
                        Some(err) => {
                            Err(TransactionError::AccountNotFound) //TODO: This is bad
                        }
                        None => Ok(()),
                    };
                    let inner_ixs: Vec<InnerInstructions> = tx_meta
                        .clone()
                        .inner_instructions
                        .clone()
                        .iter()
                        .enumerate()
                        .map(|(inner_ix_index, inner_ix)| InnerInstructions {
                            index: inner_ix_index as u8,
                            instructions: inner_ix
                                .iter()
                                .map(|ix| InnerInstruction {
                                    instruction: ix.instruction.clone(),
                                    stack_height: Some(ix.stack_height.into()),
                                })
                                .collect(),
                        })
                        .collect();

                    let confirmed_tx = ConfirmedTransactionWithStatusMeta {
                        slot: slot.block_height,
                        tx_with_meta: TransactionWithStatusMeta::Complete(
                            VersionedTransactionWithStatusMeta {
                                transaction: versioned_transaction,
                                meta: TransactionStatusMeta {
                                    status: status,
                                    fee: tx_meta.fee,
                                    pre_balances: tx_meta.pre_balances.clone(),
                                    post_balances: tx_meta.post_balances.clone(),
                                    inner_instructions: Some(inner_ixs),
                                    log_messages: Some(tx_meta.log_messages.clone()),
                                    pre_token_balances: tx_meta.pre_token_balances.clone().map(
                                        |balances| {
                                            balances
                                                .into_iter()
                                                .map(|b| TransactionTokenBalance {
                                                    account_index: b.account_index,
                                                    mint: b.mint,
                                                    ui_token_amount: b.ui_token_amount,
                                                    owner: b.owner,
                                                    program_id: b.program_id,
                                                })
                                                .collect::<Vec<_>>()
                                        },
                                    ),
                                    post_token_balances: tx_meta.post_token_balances.clone().map(
                                        |balances| {
                                            balances
                                                .into_iter()
                                                .map(|b| TransactionTokenBalance {
                                                    account_index: b.account_index,
                                                    mint: b.mint,
                                                    ui_token_amount: b.ui_token_amount,
                                                    owner: b.owner,
                                                    program_id: b.program_id,
                                                })
                                                .collect::<Vec<_>>()
                                        },
                                    ),
                                    rewards: None,
                                    loaded_addresses: LoadedAddresses {
                                        writable: vec![], //TODO
                                        readonly: vec![], //TODO
                                    },
                                    return_data: None,
                                    compute_units_consumed: Some(tx_meta.compute_units_consumed),
                                },
                            },
                        ),
                        block_time: None,
                    };

                    let account_metas = transaction
                        .message()
                        .account_keys
                        .iter()
                        .enumerate()
                        .map(|(idx, key)| AccountMeta {
                            pubkey: key.to_owned(),
                            is_signer: transaction.message().is_signer(idx),
                            is_writable: transaction.message().is_maybe_writable(idx, None),
                        })
                        .collect::<Vec<AccountMeta>>();
                    match encode_transaction(confirmed_tx) {
                        Ok(encoded_tx) => {
                            let mut val = serde_json::json!(encoded_tx);
                            if let Some(obj) = val.as_object_mut() {
                                let mut meta = obj
                                    .get("meta")
                                    .cloned()
                                    .unwrap_or_else(|| serde_json::json!({}))
                                    .as_object_mut()
                                    .cloned()
                                    .unwrap_or_default();

                                // Remove the "err" field if it exists
                                meta.remove("err");

                                // Add the new "err" value
                                meta.insert("err".to_string(), serde_json::json!(tx_meta.err));

                                if tx_meta.err.is_some() {
                                    meta.insert(
                                        "status".to_string(),
                                        serde_json::json!({
                                            "Err": tx_meta.err.unwrap()
                                        }),
                                    );
                                }

                                // Reinsert the updated meta object into val
                                obj.insert("meta".to_string(), serde_json::Value::Object(meta));
                            }
                            Ok(val)
                        }
                        Err(e) => Err(serde_json::json!({
                            "code": -32002,
                            "message": e.to_string(),
                        })),
                    }
                }
                None => Ok(serde_json::json!({
                    "context": { "slot": slot.block_height,"apiVersion":"2.1.13" },
                    "value": null,
                })),
            }
        }
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
