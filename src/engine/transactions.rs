use solana_sdk::{
    inner_instruction::InnerInstructionsList, signature::Signature, transaction::TransactionError,
    transaction_context::TransactionReturnData,
};

pub struct TransactionMetadata {
    pub signature: Signature,
    pub err: Option<TransactionError>,
    pub logs: Vec<String>,
    pub inner_instructions: InnerInstructionsList,
    pub compute_units_consumed: u64,
    pub return_data: TransactionReturnData,
}
