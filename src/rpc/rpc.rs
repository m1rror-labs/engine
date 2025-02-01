use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use litesvm::LiteSVM;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use super::{
    get_account_info::get_account_info, get_balance::get_balance, get_health::get_health,
    get_minimum_balance_for_rent_exemption::get_minimum_balance_for_rent_exemption,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RpcMethod {
    GetAccountInfo,
    GetBalance,
    GetBlock,
    GetBlockCommitment,
    GetBlockHeight,
    GetBlockProduction,
    GetBlocks,
    GetBlocksWithLimit,
    GetBlockTime,
    GetClusterNodes,
    GetEpochInfo,
    GetEpochSchedule,
    GetFeeForMessage,
    GetFirstAvailableBlock,
    GetGenesisHash,
    GetHealth,
    GetHighestSnapshotSlot,
    GetIdentity,
    GetInflationGovernor,
    GetInflationRate,
    GetInflationReward,
    GetLargestAccounts,
    GetLatestBlockhash,
    GetLeaderSchedule,
    GetMaxRetransmitSlot,
    GetMaxShredInsertSlot,
    GetMinimumBalanceForRentExemption,
    GetMultipleAccounts,
    GetProgramAccounts,
    GetRecentPerformanceSamples,
    GetRecentPrioritizationFees,
    GetSignaturesForAddress,
    GetSignatureStatuses,
    GetSlot,
    GetSlotLeader,
    GetSlotLeaders,
    GetStakeMinimumDelegation,
    GetSupply,
    GetTokenAccountBalance,
    GetTokenAccountsByDelegate,
    GetTokenAccountsByOwner,
    GetTokenLargestAccounts,
    GetTokenSupply,
    GetTransaction,
    GetTransactionCount,
    GetVersion,
    GetVoteAccounts,
    IsBlockhashValid,
    MinimumLedgerSlot,
    RequestAirdrop,
    SendTransaction,
    SimulateTransaction,
}

#[derive(Deserialize, Debug)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: RpcMethod,
    pub params: Option<serde_json::Value>,
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
        RpcMethod::GetBlock => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetBlockCommitment => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetBlockHeight => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetBlockProduction => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetBlocks => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetBlocksWithLimit => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetBlockTime => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetClusterNodes => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetEpochInfo => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetEpochSchedule => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetFeeForMessage => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetFirstAvailableBlock => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetGenesisHash => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetHealth => get_health(),
        RpcMethod::GetHighestSnapshotSlot => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetIdentity => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetInflationGovernor => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetInflationRate => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetInflationReward => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetLargestAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetLatestBlockhash => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetLeaderSchedule => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetMaxRetransmitSlot => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetMaxShredInsertSlot => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetMinimumBalanceForRentExemption => {
            get_minimum_balance_for_rent_exemption(&req, deps)
        }
        RpcMethod::GetMultipleAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetProgramAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetRecentPerformanceSamples => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetRecentPrioritizationFees => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSignaturesForAddress => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSignatureStatuses => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSlot => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSlotLeader => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSlotLeaders => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetStakeMinimumDelegation => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSupply => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenAccountBalance => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenAccountsByDelegate => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenAccountsByOwner => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenLargestAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenSupply => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTransaction => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTransactionCount => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetVersion => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetVoteAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::IsBlockhashValid => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::MinimumLedgerSlot => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::RequestAirdrop => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::SendTransaction => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::SimulateTransaction => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
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
