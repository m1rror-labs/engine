use std::sync::Arc;

use blocks::Block;
use solana_compute_budget::compute_budget::ComputeBudget;
use solana_sdk::{
    account::Account, feature_set::FeatureSet, fee::FeeStructure, hash::Hash, signature::Keypair,
};

pub mod blocks;

pub trait SVM {
    fn new() -> Self;
    fn account(&self) -> Result<Account, String>;
    fn block(&self) -> Result<Block, String>;
}

// pub struct SvmEngine {
//     accounts: AccountsDb,
//     airdrop_kp: Keypair,
//     feature_set: Arc<FeatureSet>,
//     latest_blockhash: Hash,
//     history: TransactionHistory,
//     compute_budget: Option<ComputeBudget>,
//     sigverify: bool,
//     blockhash_check: bool,
//     fee_structure: FeeStructure,
//     log_bytes_limit: Option<usize>,
// }
