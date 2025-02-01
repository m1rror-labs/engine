use solana_sdk::{hash::Hash, transaction::VersionedTransaction};

pub struct Block {
    pub blockhash: Hash,          // Hash of this block
    pub previous_blockhash: Hash, // Hash of the block preceding this block
    pub block_height: u64,        // Number of blocks from the genesis block
    pub block_time: u64,          // Unix timestamp
    pub parent_slot: u64,         // Slot of the block preceding this block
    pub transactions: Vec<VersionedTransaction>,
}
