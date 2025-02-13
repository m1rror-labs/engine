use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_signature, RpcRequest};

pub fn get_transaction<T: Storage + Clone>(
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
            Some((transaction, status)) => Ok(serde_json::json!({
                "context": { "slot": 341197053 },
                "value": {
                    "slot": status.slot,
                    "transaction": {
                        "message": {
                            "accountKeys": transaction.message.account_keys.iter().map(|key| key.to_string()).collect::<Vec<String>>(),
                            "header": {
                                "numReadonlySignedAccounts": transaction.message.header.num_readonly_signed_accounts,
                                "numReadonlyUnsignedAccounts": transaction.message.header.num_readonly_unsigned_accounts,
                                "numRequiredSignatures": transaction.message.header.num_required_signatures,
                            },
                            "instructions": transaction.message.instructions.iter().map(|instruction| {
                                serde_json::json!({
                                    "accounts": instruction.accounts,
                                    "data": instruction.data,
                                    "programIdIndex": instruction.program_id_index,
                                })
                            }).collect::<Vec<Value>>(),
                            "recentBlockhash": transaction.message.recent_blockhash.to_string(),
                        },
                        "signatures": transaction.signatures.iter().map(|signature| signature.to_string()).collect::<Vec<String>>(),
                    },
                },
            })),
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
