use std::ops::Deref;

use serde_json::Value;
use solana_account_decoder::{
    parse_account_data::{AccountAdditionalDataV2, SplTokenAdditionalData},
    parse_token::is_known_spl_token_id,
};
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::RpcFilterType,
};
use spl_token_2022::{extension::StateWithExtensions, state::Account as TokenAccount};
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{encode_account, parse_pubkey, RpcRequest};

pub fn get_program_accounts<T: Storage + Clone + 'static>(
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
    let config: Option<RpcProgramAccountsConfig> = req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_object())
        .map(|map| serde_json::from_value(Value::Object(map.clone())))
        .transpose()
        .unwrap_or_default();
    let RpcProgramAccountsConfig {
        filters,
        account_config,
        with_context,
        sort_results,
    } = config.unwrap_or_default();

    let encoding = account_config.encoding.unwrap_or(UiAccountEncoding::Base64);

    let slot = match svm.get_latest_block(id) {
        Ok(slot) => slot,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32002,
                "message": "Failed to get latest block",
            }))
        }
    };

    match svm.get_program_accounts(id, &pubkey) {
        Ok(accounts) => Ok(serde_json::Value::Array(
            accounts
                .iter()
                .filter(|(_, account)| {
                    if let Some(filters) = &filters {
                        for filter in filters {
                            match filter {
                                RpcFilterType::DataSize(data_size) => {
                                    if account.data.len() as u64 != *data_size {
                                        return false;
                                    }
                                }
                                _ => {
                                    // Handle other filter types if needed
                                }
                            }
                        }
                    }
                    true
                })
                .map(|(pubkey, account)| {
                    let additional_data = match is_known_spl_token_id(&account.owner) {
                        true => match StateWithExtensions::<TokenAccount>::unpack(&account.data) {
                            Ok(token_account) => {
                                match svm.get_mint_data_sync(id, &token_account.base.mint) {
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

                    let account_data =
                        match encode_account(account, pubkey, encoding, additional_data, None) {
                            Ok(data) => data,
                            Err(_) => return serde_json::json!(null),
                        };
                    serde_json::json!({
                        "pubkey": pubkey.to_string(),
                        "account": {
                            "data": account_data.data,
                            "executable": account.executable,
                            "lamports": account.lamports,
                            "owner": account.owner.to_string(),
                            "rentEpoch": account.rent_epoch,
                        },
                    })
                })
                .collect::<Vec<_>>(),
        )),
        Err(e) => Err(serde_json::json!({
            "code": -32002,
            "message": e,
        })),
    }
}
