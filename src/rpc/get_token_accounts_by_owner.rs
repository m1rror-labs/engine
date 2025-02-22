use serde_json::Value;
use solana_sdk::program_pack::Pack;
use spl_token::state::Account as SplAccount;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub fn get_token_accounts_by_owner<T: Storage + Clone + 'static>(
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
    let pubkey = match parse_pubkey(pubkey_str) {
        Ok(pubkey) => pubkey,
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

    match svm.get_token_accounts_by_owner(id, &pubkey) {
        Ok(accounts) => {
            let vals = accounts
                .iter()
                .map(|(pubkey, account)| {
                    let mint =
                        SplAccount::unpack_from_slice(account.data.as_slice()).map_err(|e| {
                            Err(serde_json::json!({
                                "code": -32002,
                                "message": e.to_string(),
                            }))
                        });

                    let mint = match mint {
                        Ok(mint) => mint,
                        Err(e) => return e,
                    };

                    Ok(serde_json::json!({
                        "account": {
                            "data": {
                              "parsed": {
                                "info": {
                                  "isNative": mint.is_native(),
                                  "mint": mint.mint.to_string(),
                                  "owner": mint.owner.to_string(),
                                  "state": "initialized",
                                  "tokenAmount": {
                                    "amount": mint.amount,
                                    "decimals": "", //TODO: Implement this
                                    "uiAmount": mint.amount,
                                    "uiAmountString": mint.amount.to_string(),
                                  }
                                },
                                "type": "account"
                              },
                              "program": "spl-token",
                              "space": account.data.len()
                            },
                            "executable": account.executable,
                            "lamports": account.lamports,
                            "owner": account.owner.to_string(),
                            "rentEpoch": account.rent_epoch,
                            "space": account.data.len(),
                          },
                          "pubkey": pubkey.to_string(),
                    }))
                })
                .collect::<Result<Value, Value>>();

            let vals = match vals {
                Ok(vals) => vals,
                Err(e) => return Err(e),
            };

            Ok(serde_json::json!({
                "context": { "apiVersion":"2.1.13", "slot": slot.block_height },
                "value": vals}))
        }
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
