use serde_json::Value;
use solana_sdk::program_pack::Pack;
use spl_token::state::Account as SplAccount;
use spl_token_2022::{extension::StateWithExtensions, state::Mint};
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
            }));
        }
    };

    match svm.get_token_accounts_by_owner(id, &pubkey) {
        Ok(accounts) => {
            let vals = accounts
                .iter()
                .map(|(pubkey, account)| {
                    let ata = SplAccount::unpack_from_slice(account.data.as_slice()).map_err(|e| {
                        Err(serde_json::json!({
                            "code": -32002,
                            "message": e.to_string(),
                        }))
                    });
                    let ata = match ata {
                        Ok(ata) => ata,
                        Err(e) => return e,
                    };

                    let mint_account = match svm.get_account(id, &ata.mint) {
                        Ok(mint) => match mint {
                            Some(mint) => mint,
                            None => {
                                return Err(serde_json::json!({
                                    "code": -32002,
                                    "message": "Mint account not found",
                                }));
                            }
                        },
                        Err(e) => {
                            return Err(serde_json::json!({
                                "code": -32002,
                                "message": e.to_string(),
                            }));
                        }
                    };

                    let mint = match StateWithExtensions::<Mint>::unpack(&mint_account.data).ok() {
                        Some(token_account) => token_account,
                        None => {
                            return Err(serde_json::json!({
                                "code": -32002,
                                "message": "Failed to unpack token account",
                            }));
                        }
                    };
                    let ui_amount = ata.amount as f64 / 10f64.powi(mint.base.decimals as i32);

                    Ok(serde_json::json!({
                        "account": {
                            "data": {
                              "parsed": {
                                "info": {
                                  "isNative": ata.is_native(),
                                  "mint": ata.mint.to_string(),
                                  "owner": ata.owner.to_string(),
                                  "state": "initialized",
                                  "tokenAmount": {
                                    "amount": ata.amount.to_string(),
                                    "decimals": mint.base.decimals,
                                    "uiAmount": ui_amount,
                                    "uiAmountString": ui_amount.to_string(),
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
