use serde_json::Value;
use solana_sdk::account::AccountSharedData;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_tx, RpcRequest};

pub fn simulate_transaction<T: Storage + Clone>(
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

    match svm.simulate_transaction(id, tx) {
        Ok(res) => {
            let return_data_str = base64::encode(&res.return_data.data);
            Ok(serde_json::json!({
                "context": {
                    "slot": 218
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
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Failed to send tx",
        })),
    }
}
