use std::{cmp::min, fmt, str::FromStr};

use base64::prelude::*;
use bincode::Options;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_account_decoder::{encode_ui_account, parse_account_data::AccountAdditionalDataV2};
use solana_account_decoder_client_types::{UiAccount, UiAccountEncoding, UiDataSliceConfig};
use solana_sdk::{
    account::ReadableAccount, bs58, hash::Hash, packet::PACKET_DATA_SIZE, pubkey::Pubkey,
    signature::Signature, transaction::VersionedTransaction,
};
use solana_transaction_status_client_types::TransactionBinaryEncoding;
use std::any::type_name;
use uuid::Uuid;

use crate::{engine::SvmEngine, storage::Storage};

use super::{
    get_account_info::get_account_info, get_balance::get_balance, get_block::get_block,
    get_block_commitment::get_block_commitment, get_block_height::get_block_height,
    get_block_time::get_block_time, get_epoch_info::get_epoch_info,
    get_genesis_hash::get_genesis_hash, get_health::get_health, get_identity::get_identity,
    get_largest_accounts::get_largest_accounts, get_latest_blockhash::get_latest_blockhash,
    get_minimum_balance_for_rent_exemption::get_minimum_balance_for_rent_exemption,
    get_multiple_accounts::get_multiple_accounts, get_program_accounts::get_program_accounts,
    get_signature_statuses::get_signature_statuses,
    get_signatures_for_address::get_signatures_for_address, get_slot_leaders::get_slot_leaders,
    get_token_account_balance::get_token_account_balance,
    get_token_accounts_by_owner::get_token_accounts_by_owner, get_token_supply::get_token_supply,
    get_transaction::get_transaction, get_transaction_count::get_transaction_count,
    get_version::get_version, is_blockhash_valid::is_blockhash_valid,
    request_airdrop::request_airdrop, send_transaction::send_transaction,
    simulate_transaction::simulate_transaction,
};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub async fn handle_request<T: Storage + Clone + 'static>(
    id: Uuid,
    req: RpcRequest,
    svm: &SvmEngine<T>,
) -> RpcResponse {
    let result = match req.method {
        RpcMethod::GetAccountInfo => get_account_info(id, &req, svm).await,
        RpcMethod::GetBalance => get_balance(id, &req, svm).await,
        RpcMethod::GetBlock => get_block(id, &req, svm),
        RpcMethod::GetBlockCommitment => get_block_commitment(id, &req, svm),
        RpcMethod::GetBlockHeight => get_block_height(id, svm),
        RpcMethod::GetBlockProduction => Ok(serde_json::json!({
                "context": {
                  "slot": 9887,"apiVersion":"2.1.13"
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
        RpcMethod::GetBlockTime => get_block_time(id, &req, svm),
        RpcMethod::GetClusterNodes => Ok(serde_json::json!([])),
        RpcMethod::GetEpochInfo => get_epoch_info(id, svm),
        RpcMethod::GetEpochSchedule => Ok(serde_json::json!({
                "firstNormalEpoch": 8,
                "firstNormalSlot": 8160,
                "leaderScheduleSlotOffset": 8192,
                "slotsPerEpoch": 8192,
                "warmup": true
        })),
        RpcMethod::GetFeeForMessage => Ok(serde_json::json!({
            "context": { "slot": 5068,"apiVersion":"2.1.13" }, "value": 5000
        })),
        RpcMethod::GetFirstAvailableBlock => Ok(serde_json::json!(1)),
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
        RpcMethod::GetLargestAccounts => get_largest_accounts(id, svm),
        RpcMethod::GetLatestBlockhash => get_latest_blockhash(id, svm),
        RpcMethod::GetLeaderSchedule => Ok(serde_json::json!(null)),
        RpcMethod::GetMaxRetransmitSlot => get_block_height(id, svm),
        RpcMethod::GetMaxShredInsertSlot => get_block_height(id, svm),
        RpcMethod::GetMinimumBalanceForRentExemption => {
            get_minimum_balance_for_rent_exemption(&req, svm)
        }
        RpcMethod::GetMultipleAccounts => get_multiple_accounts(id, &req, svm).await,
        RpcMethod::GetProgramAccounts => get_program_accounts(id, &req, svm),
        RpcMethod::GetRecentPerformanceSamples => Ok(serde_json::json!([{
          "numSlots": 126,
          "numTransactions": 126,
          "numNonVoteTransactions": 1,
          "samplePeriodSecs": 60,
          "slot": 348125
        }])),
        RpcMethod::GetRecentPrioritizationFees => Ok(serde_json::json!([{
          "slot": 348125,
          "prioritizationFee": 0
        }])),
        RpcMethod::GetSignaturesForAddress => get_signatures_for_address(id, &req, svm),
        RpcMethod::GetSignatureStatuses => get_signature_statuses(id, &req, svm),
        RpcMethod::GetSlot => get_block_height(id, svm),
        RpcMethod::GetSlotLeader => get_identity(id, svm),
        RpcMethod::GetSlotLeaders => get_slot_leaders(id, &req, svm),
        RpcMethod::GetStakeMinimumDelegation => Err(serde_json::json!({
            "context": {
                "slot": 501,"apiVersion":"2.1.13"
              },
              "value": 1000000000
        })),
        //TODO: fix this
        RpcMethod::GetSupply => Ok(serde_json::json!({
            "context": {
                "slot": 1114,"apiVersion":"2.1.13"
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
        RpcMethod::GetTokenAccountBalance => get_token_account_balance(id, &req, svm).await,
        RpcMethod::GetTokenAccountsByDelegate => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenAccountsByOwner => get_token_accounts_by_owner(id, &req, svm).await,
        RpcMethod::GetTokenLargestAccounts => Err(serde_json::json!({
            "code": -32601,
            "message": "Method not found",
        })),
        RpcMethod::GetTokenSupply => get_token_supply(id, &req, svm).await,
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
        RpcMethod::RequestAirdrop => request_airdrop(id, &req, svm).await,
        RpcMethod::SendTransaction => send_transaction(id, &req, svm).await,
        RpcMethod::SimulateTransaction => simulate_transaction(id, &req, svm).await,
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

pub fn encode_account<T: ReadableAccount>(
    account: &T,
    pubkey: &Pubkey,
    encoding: UiAccountEncoding,
    additional_data: Option<AccountAdditionalDataV2>,
    data_slice: Option<UiDataSliceConfig>,
) -> Result<UiAccount, String> {
    if (encoding == UiAccountEncoding::Binary || encoding == UiAccountEncoding::Base58)
        && data_slice
            .map(|s| min(s.length, account.data().len().saturating_sub(s.offset)))
            .unwrap_or(account.data().len())
            > MAX_BASE58_SIZE
    {
        let message = format!("Encoded binary (base 58) data should be less than {MAX_BASE58_SIZE} bytes, please use Base64 encoding.");
        Err(message)
    } else {
        Ok(encode_ui_account(
            pubkey,
            account,
            encoding,
            additional_data,
            data_slice,
        ))
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
    let tx_data = BASE64_STANDARD.decode(tx_str.as_str().unwrap().as_bytes());
    let tx_data = match tx_data {
        Ok(tx_data) => tx_data,
        Err(_) => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "Invalid params: unable to parse tx",
            }));
        }
    };

    match bincode::deserialize(&tx_data) {
        Ok(tx) => Ok(tx),
        Err(_) => Err(serde_json::json!({
            "code": -32602,
            "message": "Invalid params: unable to parse tx",
        })),
    }
}

const MAX_BASE58_SIZE: usize = 1683; // Golden, bump if PACKET_DATA_SIZE changes
const MAX_BASE64_SIZE: usize = 1644; // Golden, bump if PACKET_DATA_SIZE changes
pub fn decode_and_deserialize<T>(
    encoded: String,
    encoding: TransactionBinaryEncoding,
) -> Result<(Vec<u8>, T), String>
where
    T: serde::de::DeserializeOwned,
{
    let wire_output = match encoding {
        TransactionBinaryEncoding::Base58 => {
            if encoded.len() > MAX_BASE58_SIZE {
                return Err(format!(
                    "base58 encoded {} too large: {} bytes (max: encoded/raw {}/{})",
                    type_name::<T>(),
                    encoded.len(),
                    MAX_BASE58_SIZE,
                    PACKET_DATA_SIZE,
                ));
            }
            bs58::decode(encoded)
                .into_vec()
                .map_err(|e| format!("invalid base58 encoding: {e:?}"))?
        }
        TransactionBinaryEncoding::Base64 => {
            if encoded.len() > MAX_BASE64_SIZE {
                return Err(format!(
                    "base64 encoded {} too large: {} bytes (max: encoded/raw {}/{})",
                    type_name::<T>(),
                    encoded.len(),
                    MAX_BASE64_SIZE,
                    PACKET_DATA_SIZE,
                ));
            }
            BASE64_STANDARD
                .decode(encoded)
                .map_err(|e| format!("invalid base64 encoding: {e:?}"))?
        }
    };
    if wire_output.len() > PACKET_DATA_SIZE {
        return Err(format!(
            "decoded {} too large: {} bytes (max: {} bytes)",
            type_name::<T>(),
            wire_output.len(),
            PACKET_DATA_SIZE
        ));
    }
    bincode::options()
        .with_limit(PACKET_DATA_SIZE as u64)
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .deserialize_from(&wire_output[..])
        .map_err(|err| {
            format!(
                "failed to deserialize {}: {}",
                type_name::<T>(),
                &err.to_string()
            )
        })
        .map(|output| (wire_output, output))
}
