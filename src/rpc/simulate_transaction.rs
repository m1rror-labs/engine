use base64::prelude::*;
use serde_json::Value;
use solana_sdk::{account::AccountSharedData, bpf_loader, bpf_loader_upgradeable};
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_tx, RpcRequest};

pub async fn simulate_transaction<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let tx = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| Some(v))
    {
        Some(s) => match parse_tx(s.clone()) {
            Ok(tx) => tx,
            Err(_) => {
                return Err(serde_json::json!({
                    "code": -32602,
                    "message": "Invalid params: unable to parse tx"
                }));
            }
        },
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
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

    if tx
        .message
        .instructions()
        .iter()
        .map(|ix| ix.program_id(tx.message.static_account_keys()))
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

    let blockchain = match svm.storage.get_blockchain(id) {
        Ok(blockchain) => blockchain,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm.simulate_transaction(id, tx, blockchain.jit).await {
        Ok(res) => {
            let return_data_str = BASE64_STANDARD.encode(&res.return_data.data);
            Ok(serde_json::json!({
                "context": {
                    "slot": slot.block_height,"apiVersion":"2.1.13"
                  },
                  "value": {
                    "err": res.err,
                    "accounts": res.post_accounts.iter().map(|(_, account)|  {
                        account
                    }).collect::<Vec<&AccountSharedData>>(),
                    "logs": res.logs,
                    "returnData": {
                      "data": [return_data_str, "base64"],
                      "programId": res.return_data.program_id.to_string(),
                    },
                    "unitsConsumed": res.compute_units_consumed,
                  }
            }))
        }
        Err(e) => Err(serde_json::json!({
            "code": -32602,
            "message": e,
        })),
    }
}
