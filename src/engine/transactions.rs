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
