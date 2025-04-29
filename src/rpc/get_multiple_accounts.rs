use serde_json::Value;
use solana_account_decoder::{
    parse_account_data::{AccountAdditionalDataV2, SplTokenAdditionalData},
    parse_token::is_known_spl_token_id,
    UiAccountEncoding,
};
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{extension::StateWithExtensions, state::Account as TokenAccount};
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    rpc::rpc::encode_account,
    storage::Storage,
};

use super::rpc::{parse_pubkey, RpcRequest};

pub async fn get_multiple_accounts<T: Storage + Clone + 'static>(
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
    let config: Option<RpcAccountInfoConfig> = req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_object())
        .map(|map| serde_json::from_value(Value::Object(map.clone())))
        .transpose()
        .unwrap_or_default();
    let RpcAccountInfoConfig {
        encoding,
        data_slice,
        commitment,
        min_context_slot,
    } = config.unwrap_or_default();
    _ = commitment;
    _ = min_context_slot;

    let encoding = encoding.unwrap_or(UiAccountEncoding::Base64);

    let blockchain = match svm.storage.get_blockchain(id) {
        Ok(blockchain) => blockchain,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm
        .get_multiple_accounts(id, &pubkeys, blockchain.jit)
        .await
    {
        Ok(accounts) => Ok(serde_json::json!({
            "context": { "apiVersion":"2.1.13", "slot": 341197247 },
            "value": accounts
            .iter()
            .enumerate()
            .map(|(idx, account)| match account {
                Some(account) => {
                    let additional_data = match is_known_spl_token_id(&account.owner) {
                        true => match StateWithExtensions::<TokenAccount>::unpack(&account.data) {
                            Ok(token_account) => {
                                match svm
                                    .get_mint_data_sync(id, &token_account.base.mint)

                                {
                                    Ok(mint_data) => Some(AccountAdditionalDataV2 {
                                        spl_token_additional_data: Some(SplTokenAdditionalData {
                                            decimals: mint_data.decimals,
                                            interest_bearing_config: None,
                                        }),
                                    }),
                                    Err(_) => None,
                                }
                            }
                            Err(_) => None,
                        },
                        false => None,
                    };

                    let account_data = match encode_account(
                        account,
                        &pubkeys[idx],
                        encoding,
                        additional_data,
                        data_slice,
                    ) {
                        Ok(data) => data,
                        Err(_) => {
                            return serde_json::json!(null)
                        }
                    };
                    serde_json::json!(account_data)
                },
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
