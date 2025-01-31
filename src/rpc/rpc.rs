use std::sync::{Arc, RwLock};

use litesvm::LiteSVM;
use serde::{Deserialize, Serialize};

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
    match req.method {
        RpcMethod::GetAccountInfo => get_account_info(req, deps),
        RpcMethod::GetBalance => get_balance(req),
    }
}
