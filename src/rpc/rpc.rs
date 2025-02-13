use std::{fmt, str::FromStr};

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
    get_multiple_accounts::get_multiple_accounts, get_signature_statuses::get_signature_statuses,
    get_signatures_for_address::get_signatures_for_address,
    get_token_account_balance::get_token_account_balance,
    get_token_accounts_by_owner::get_token_accounts_by_owner, get_token_supply::get_token_supply,
    get_transaction::get_transaction, get_transaction_count::get_transaction_count,
    get_version::get_version, is_blockhash_valid::is_blockhash_valid,
    request_airdrop::request_airdrop, send_transaction::send_transaction,
    simulate_transaction::simulate_transaction,
};

#[derive(Deserialize, Debug, Clone, Copy)]
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

impl fmt::Display for RpcMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method_str = match self {
            RpcMethod::GetAccountInfo => "GetAccountInfo",
            RpcMethod::GetBalance => "GetBalance",
            RpcMethod::GetBlock => "GetBlock",
            RpcMethod::GetBlockCommitment => "GetBlockCommitment",
            RpcMethod::GetBlockHeight => "GetBlockHeight",
            RpcMethod::GetBlockProduction => "GetBlockProduction",
            RpcMethod::GetBlocks => "GetBlocks",
            RpcMethod::GetBlocksWithLimit => "GetBlocksWithLimit",
            RpcMethod::GetBlockTime => "GetBlockTime",
            RpcMethod::GetClusterNodes => "GetClusterNodes",
            RpcMethod::GetEpochInfo => "GetEpochInfo",
            RpcMethod::GetEpochSchedule => "GetEpochSchedule",
            RpcMethod::GetFeeForMessage => "GetFeeForMessage",
            RpcMethod::GetFirstAvailableBlock => "GetFirstAvailableBlock",
            RpcMethod::GetGenesisHash => "GetGenesisHash",
            RpcMethod::GetHealth => "GetHealth",
            RpcMethod::GetHighestSnapshotSlot => "GetHighestSnapshotSlot",
            RpcMethod::GetIdentity => "GetIdentity",
            RpcMethod::GetInflationGovernor => "GetInflationGovernor",
            RpcMethod::GetInflationRate => "GetInflationRate",
            RpcMethod::GetInflationReward => "GetInflationReward",
            RpcMethod::GetLargestAccounts => "GetLargestAccounts",
            RpcMethod::GetLatestBlockhash => "GetLatestBlockhash",
            RpcMethod::GetLeaderSchedule => "GetLeaderSchedule",
            RpcMethod::GetMaxRetransmitSlot => "GetMaxRetransmitSlot",
            RpcMethod::GetMaxShredInsertSlot => "GetMaxShredInsertSlot",
            RpcMethod::GetMinimumBalanceForRentExemption => "GetMinimumBalanceForRentExemption",
            RpcMethod::GetMultipleAccounts => "GetMultipleAccounts",
            RpcMethod::GetProgramAccounts => "GetProgramAccounts",
            RpcMethod::GetRecentPerformanceSamples => "GetRecentPerformanceSamples",
            RpcMethod::GetRecentPrioritizationFees => "GetRecentPrioritizationFees",
            RpcMethod::GetSignaturesForAddress => "GetSignaturesForAddress",
            RpcMethod::GetSignatureStatuses => "GetSignatureStatuses",
            RpcMethod::GetSlot => "GetSlot",
            RpcMethod::GetSlotLeader => "GetSlotLeader",
            RpcMethod::GetSlotLeaders => "GetSlotLeaders",
            RpcMethod::GetStakeMinimumDelegation => "GetStakeMinimumDelegation",
            RpcMethod::GetSupply => "GetSupply",
            RpcMethod::GetTokenAccountBalance => "GetTokenAccountBalance",
            RpcMethod::GetTokenAccountsByDelegate => "GetTokenAccountsByDelegate",
            RpcMethod::GetTokenAccountsByOwner => "GetTokenAccountsByOwner",
            RpcMethod::GetTokenLargestAccounts => "GetTokenLargestAccounts",
            RpcMethod::GetTokenSupply => "GetTokenSupply",
            RpcMethod::GetTransaction => "GetTransaction",
            RpcMethod::GetTransactionCount => "GetTransactionCount",
            RpcMethod::GetVersion => "GetVersion",
            RpcMethod::GetVoteAccounts => "GetVoteAccounts",
            RpcMethod::IsBlockhashValid => "IsBlockhashValid",
            RpcMethod::MinimumLedgerSlot => "MinimumLedgerSlot",
            RpcMethod::RequestAirdrop => "RequestAirdrop",
            RpcMethod::SendTransaction => "SendTransaction",
            RpcMethod::SimulateTransaction => "SimulateTransaction",
            RpcMethod::GetAsset => "GetAsset",
        };
        write!(f, "{}", method_str)
    }
}

#[derive(Deserialize, Debug, Clone)]
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
        RpcMethod::GetSignatureStatuses => get_signature_statuses(id, &req, svm),
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
        RpcMethod::GetTokenAccountBalance => get_token_account_balance(id, &req, svm),
        RpcMethod::GetTokenAccountsByDelegate => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenAccountsByOwner => get_token_accounts_by_owner(id, &req, svm),
        RpcMethod::GetTokenLargestAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenSupply => get_token_supply(id, &req, svm),
        RpcMethod::GetTransaction => get_transaction(id, &req, svm),
        RpcMethod::GetTransactionCount => get_transaction_count(id, svm),
        RpcMethod::GetVersion => get_version(),
        RpcMethod::GetVoteAccounts => Ok(serde_json::json!({
            "current": [
                {
                  "commission": 0,
                  "epochVoteAccount": true,
                  "epochCredits": [
                    [1, 64, 0],
                    [2, 192, 64]
                  ],
                  "nodePubkey": "B97CCUW3AEZFGy6uUg6zUdnNYvnVq5VG8PUtb2HayTDD",
                  "lastVote": 147,
                  "activatedStake": 42,
                  "votePubkey": "3ZT31jkAGhUaw8jsy4bTknwBMP8i4Eueh52By4zXcsVw"
                }
              ],
              "delinquent": []
        })),
        RpcMethod::IsBlockhashValid => is_blockhash_valid(id, &req, svm),
        RpcMethod::MinimumLedgerSlot => Ok(serde_json::json!(0)),
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
