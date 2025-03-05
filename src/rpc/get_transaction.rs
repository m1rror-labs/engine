use serde_json::Value;
use solana_sdk::{bs58, instruction::AccountMeta};
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

    let slot = match svm.get_latest_block(id) {
        Ok(slot) => slot,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm.get_transaction(id, &signature) {
        Ok(transaction) => match transaction {
            Some((transaction, tx_meta, status)) => {
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
                Ok(serde_json::json!({
                    "slot": slot.block_height,
                        "blockTime": slot.block_time,
                        "slot": status.slot,
                        "meta": tx_meta,
                        "transaction": {
                            "message": {
                                "accountKeys": account_metas.iter().map(|meta| {
                                    serde_json::json!({
                                        "pubkey": meta.pubkey.to_string(),
                                        "signer": meta.is_signer,
                                        "writable": meta.is_writable,
                                        "source": "transaction",
                                    })
                                }).collect::<Vec<Value>>(),
                                "instructions": transaction.message.instructions.iter().map(|instruction| {
                                    let program_id = instruction.program_id(&transaction.message.account_keys);
                                    let data_str = bs58::encode(&instruction.data).into_string();
                                    serde_json::json!({
                                        "accounts": instruction.accounts.iter().map(|idx| transaction.message.account_keys[*idx as usize].to_string()).collect::<Vec<String>>(),
                                        "data": data_str,
                                        "programId": program_id.to_string(),
                                        "stackHeight":null,
                                    })
                                }).collect::<Vec<Value>>(),
                                "recentBlockhash": transaction.message.recent_blockhash.to_string(),
                            },
                            "signatures": transaction.signatures.iter().map(|signature| signature.to_string()).collect::<Vec<String>>(),
                    },
                    "version": "legacy", //TODO: versioning
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
