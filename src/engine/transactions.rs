use serde::Serialize;
use serde_json::Value;
use solana_sdk::{
    account::AccountSharedData,
    inner_instruction::InnerInstructionsList,
    pubkey::Pubkey,
    signature::Signature,
    transaction::{SanitizedTransaction, TransactionError},
    transaction_context::TransactionReturnData,
};

use super::blocks::Block;

pub struct TransactionMetadata {
    pub signature: Signature,
    pub err: Option<TransactionError>,
    pub logs: Vec<String>,
    pub inner_instructions: InnerInstructionsList,
    pub compute_units_consumed: u64,
    pub return_data: TransactionReturnData,
    pub tx: SanitizedTransaction,
    pub current_block: Block,
    pub pre_accounts: Vec<(Pubkey, AccountSharedData)>,
    pub post_accounts: Vec<(Pubkey, AccountSharedData)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]

pub struct TransactionMeta {
    pub err: Option<String>,
    pub fee: u64,
    pub log_messages: Vec<String>,
    pub inner_instructions: InnerInstructionsList,
    pub compute_units_consumed: u64,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<u64>,
    pub rewards: Vec<u64>, //todo: rewards
    pub status: Value,
}
