use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use litesvm::LiteSVM;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use super::{get_account_info::get_account_info, get_balance::get_balance};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RpcMethod {
    GetAccountInfo,
    GetBalance,
}

#[derive(Deserialize, Debug)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: RpcMethod,
    pub params: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct Dependencies {
    pub lite_svm: Arc<RwLock<LiteSVM>>,
}

impl Dependencies {
    pub fn new(lite_svm: LiteSVM) -> Self {
        Self {
            lite_svm: Arc::new(RwLock::new(lite_svm)),
        }
    }
}

pub fn handle_request(req: RpcRequest, deps: &Dependencies) -> RpcResponse {
    let result = match req.method {
        RpcMethod::GetAccountInfo => get_account_info(&req, deps),
        RpcMethod::GetBalance => get_balance(&req, deps),
    };

    match result {
        Ok(r) => RpcResponse {
            jsonrpc: req.jsonrpc,
            id: req.id,
            result: Some(r),
            error: None,
        },
        Err(e) => RpcResponse {
            jsonrpc: req.jsonrpc,
            id: req.id,
            result: None,
            error: Some(e),
        },
    }
}

pub fn parse_pubkey(pubkey_str: &str) -> Result<Pubkey, Value> {
    match Pubkey::from_str(pubkey_str) {
        Ok(pk) => Ok(pk),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse pubkey",
        })),
    }
}
