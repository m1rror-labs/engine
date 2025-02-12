use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::{
    hash::Hash, pubkey::Pubkey, signature::Signature, transaction::VersionedTransaction,
};
use uuid::Uuid;

use crate::{engine::SvmEngine, storage::Storage};

use super::{
    get_account_info::get_account_info, get_balance::get_balance, get_block::get_block,
    get_block_commitment::get_block_commitment, get_block_height::get_block_height,
    get_genesis_hash::get_genesis_hash, get_health::get_health, get_identity::get_identity,
    get_latest_blockhash::get_latest_blockhash,
    get_minimum_balance_for_rent_exemption::get_minimum_balance_for_rent_exemption,
    get_multiple_accounts::get_multiple_accounts,
    get_signatures_for_address::get_signatures_for_address, get_version::get_version,
    is_blockhash_valid::is_blockhash_valid, request_airdrop::request_airdrop,
    send_transaction::send_transaction, simulate_transaction::simulate_transaction,
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

    GetAsset,
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

pub fn handle_request<T: Storage + Clone>(
    id: Uuid,
    req: RpcRequest,
    svm: &SvmEngine<T>,
) -> RpcResponse {
    let result = match req.method {
        RpcMethod::GetAccountInfo => get_account_info(id, &req, svm),
        RpcMethod::GetBalance => get_balance(id, &req, svm),
        RpcMethod::GetBlock => get_block(id, &req, svm),
        RpcMethod::GetBlockCommitment => get_block_commitment(id, &req, svm),
        RpcMethod::GetBlockHeight => get_block_height(id, svm),
        RpcMethod::GetBlockProduction => Ok(serde_json::json!({
                "context": {
                  "slot": 9887
                },
                "value": {
                  "byIdentity": {
                    "85iYT5RuzRTDgjyRa3cP8SYhM2j21fj7NhfJ3peu1DPr": [9888, 9886]
                  },
                  "range": {
                    "firstSlot": 0,
                    "lastSlot": 9887
                  }
                }
        })),
        RpcMethod::GetBlocks => Ok(serde_json::json!([5, 6, 7, 8, 9, 10])),
        RpcMethod::GetBlocksWithLimit => Ok(serde_json::json!([5, 6, 7, 8, 9, 10])),
        RpcMethod::GetBlockTime => Ok(serde_json::json!(1574721591)),
        RpcMethod::GetClusterNodes => Ok(serde_json::json!([])),
        RpcMethod::GetEpochInfo => Ok(serde_json::json!({
            "absoluteSlot": 360253902,
            "blockHeight": 348253772,
            "epoch": 833,
            "slotIndex": 397902,
            "slotsInEpoch": 432000,
            "transactionCount": 151130291,
        })),
        RpcMethod::GetEpochSchedule => Ok(serde_json::json!({
                "firstNormalEpoch": 8,
                "firstNormalSlot": 8160,
                "leaderScheduleSlotOffset": 8192,
                "slotsPerEpoch": 8192,
                "warmup": true
        })),
        RpcMethod::GetFeeForMessage => Ok(serde_json::json!({
            "context": { "slot": 5068 }, "value": 5000
        })),
        RpcMethod::GetFirstAvailableBlock => Ok(serde_json::json!(0)),
        RpcMethod::GetGenesisHash => get_genesis_hash(id, svm),
        RpcMethod::GetHealth => get_health(),
        RpcMethod::GetHighestSnapshotSlot => Err(serde_json::json!({
             "code": -32008, "message": "No snapshot"
        })),
        RpcMethod::GetIdentity => get_identity(id, svm),
        RpcMethod::GetInflationGovernor => Ok(serde_json::json!({
            "foundation": 0.05,
            "foundationTerm": 7,
            "initial": 0.15,
            "taper": 0.15,
            "terminal": 0.015
        })),
        RpcMethod::GetInflationRate => Ok(serde_json::json!({
            "epoch": 100,
            "foundation": 0.001,
            "total": 0.149,
            "validator": 0.148
        })),
        RpcMethod::GetInflationReward => Ok(serde_json::json!({
                "amount": 2500,
                "effectiveSlot": 224,
                "epoch": 2,
                "postBalance": 499999,
        })),
        RpcMethod::GetLargestAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetLatestBlockhash => get_latest_blockhash(id, svm),
        RpcMethod::GetLeaderSchedule => Ok(serde_json::json!(null)),
        RpcMethod::GetMaxRetransmitSlot => get_block_height(id, svm),
        RpcMethod::GetMaxShredInsertSlot => get_block_height(id, svm),
        RpcMethod::GetMinimumBalanceForRentExemption => {
            get_minimum_balance_for_rent_exemption(&req, svm)
        }
        RpcMethod::GetMultipleAccounts => get_multiple_accounts(id, &req, svm),
        RpcMethod::GetProgramAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetRecentPerformanceSamples => Ok(serde_json::json!([{
          "numSlots": 126,
          "numTransactions": 126,
          "numNonVoteTransactions": 1,
          "samplePeriodSecs": 60,
          "slot": 348125
        }])),
        RpcMethod::GetRecentPrioritizationFees => Err(serde_json::json!([{
          "slot": 348125,
          "prioritizationFee": 0
        }])),
        RpcMethod::GetSignaturesForAddress => get_signatures_for_address(id, &req, svm),
        RpcMethod::GetSignatureStatuses => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetSlot => get_block_height(id, svm),
        RpcMethod::GetSlotLeader => get_identity(id, svm),
        RpcMethod::GetSlotLeaders => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetStakeMinimumDelegation => Err(serde_json::json!({
            "context": {
                "slot": 501
              },
              "value": 1000000000
        })),
        //TODO: fix this
        RpcMethod::GetSupply => Ok(serde_json::json!({
            "context": {
                "slot": 1114
              },
              "value": {
                "circulating": 16000,
                "nonCirculating": 1000000,
                "nonCirculatingAccounts": [
                  "FEy8pTbP5fEoqMV1GdTz83byuA8EKByqYat1PKDgVAq5",
                  "9huDUZfxoJ7wGMTffUE7vh1xePqef7gyrLJu9NApncqA",
                  "3mi1GmwEE3zo2jmfDuzvjSX9ovRXsDUKHvsntpkhuLJ9",
                  "BYxEJTDerkaRWBem3XgnVcdhppktBXa2HbkHPKj2Ui4Z"
                ],
                "total": 1016000
              }
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
        RpcMethod::GetVersion => get_version(),
        RpcMethod::GetVoteAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::IsBlockhashValid => is_blockhash_valid(id, &req, svm),
        RpcMethod::MinimumLedgerSlot => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::RequestAirdrop => request_airdrop(id, &req, svm),
        RpcMethod::SendTransaction => send_transaction(id, &req, svm),
        RpcMethod::SimulateTransaction => simulate_transaction(id, &req, svm),
        RpcMethod::GetAsset => Err(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32000,
                    "message": "Database Error: RecordNotFound Error: Asset Not Found"
                },
                "id": "A5JxZVHgXe7fn5TqJXm6Hj2zKh1ptDapae2YjtXbZJoy"
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

pub fn parse_signature(sig_str: &str) -> Result<Signature, Value> {
    match Signature::from_str(sig_str) {
        Ok(pk) => Ok(pk),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse signature",
        })),
    }
}

pub fn parse_hash(hash_str: &str) -> Result<Hash, Value> {
    match Hash::from_str(hash_str) {
        Ok(pk) => Ok(pk),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse hash",
        })),
    }
}

pub fn parse_tx(tx_str: Value) -> Result<VersionedTransaction, Value> {
    match VersionedTransaction::deserialize(tx_str) {
        Ok(pk) => Ok(pk),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse tx",
        })),
    }
}
