use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub fn get_multiple_accounts<T: Storage + Clone>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let pubkeys_arr = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_array())
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };
    let pubkeys_str = pubkeys_arr
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<&str>>();

    let pubkeys = pubkeys_str
        .iter()
        .map(|s| parse_pubkey(s))
        .collect::<Result<Vec<Pubkey>, Value>>()?;

    let pubkeys = pubkeys.iter().map(|v| v).collect();

    match svm.get_multiple_accounts(id, &pubkeys) {
        Ok(accounts) => Ok(serde_json::json!({
            "context": { "apiVersion": "2.0.15", "slot": 341197247 },
            "value": accounts
            .iter()
            .map(|account| match account {
                Some(account) => serde_json::json!({
                        "data": [ "","base64"],
                        "executable": account.executable,
                        "lamports": account.lamports,
                        "owner": account.owner.to_string(),
                        "rentEpoch":1844674407,
                        "space": account.data.len(),
                    }
                ),
                None => serde_json::json!(null),
            })
            .collect::<Vec<_>>(),
        })),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
