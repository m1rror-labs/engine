use base64::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

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
    let pubkey = parse_pubkey(pubkey_str)?;

    match svm.get_account(id, &pubkey) {
        Ok(account) => match account {
            Some(account) => {
                let data_str = BASE64_STANDARD.encode(&account.data);
                println!("account: {:?}", account.data);
                let bytes = include_bytes!("./mpl_project_name_program.so");
                // println!("original program: {:?}", bytes);
                Ok(serde_json::json!({
                    "context": { "slot": 341197053,"apiVersion":"1.18.1" },
                    "value": {
                        "data": [ data_str,"base64"],
                        "program": [BASE64_STANDARD.encode(&bytes)],
                        "executable": account.executable,
                        "lamports": account.lamports,
                        "owner": account.owner.to_string(),
                        "rentEpoch": account.rent_epoch,
                        "space": account.data.len(),
                    },
                }))
            }
            None => Ok(serde_json::json!({
                "context": { "slot": 341197053,"apiVersion":"1.18.1" },
                "value": null,
            })),
        },
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
