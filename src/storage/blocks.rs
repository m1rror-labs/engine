use diesel::prelude::*;
use solana_sdk::{hash::Hash, signature::Keypair};
use uuid::Uuid;

use crate::engine::blocks::{Block, Blockchain};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::blockchain)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbBlockchain {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub airdrop_keypair: Vec<u8>,
}

impl DbBlockchain {
    pub fn to_blockchain(self) -> Blockchain {
        Blockchain {
            id: self.id,
            created_at: self.created_at,
            airdrop_keypair: Keypair::from_bytes(self.airdrop_keypair.as_slice()).unwrap(),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbBlock {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub blockchain: Uuid,
    pub blockhash: Vec<u8>,
    pub previous_blockhash: Vec<u8>,
    pub parent_slot: i64,
    pub block_height: i64,
    pub slot: i64,
}

impl DbBlock {
    pub fn from_block(block: &Block, blockchain: Uuid) -> Self {
        DbBlock {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_utc(),
            blockchain,
            blockhash: block.blockhash.to_bytes().to_vec(),
            previous_blockhash: block.previous_blockhash.to_bytes().to_vec(),
            parent_slot: block.parent_slot as i64,
            block_height: block.block_height as i64,
            slot: block.block_height as i64,
        }
    }

    pub fn into_block(self) -> (Block, Uuid) {
        (
            Block {
                blockhash: Hash::new_from_array(self.blockhash.as_slice().try_into().unwrap()),
                previous_blockhash: Hash::new_from_array(
                    self.previous_blockhash.as_slice().try_into().unwrap(),
                ),
                block_height: self.block_height as u64,
                block_time: self.created_at.and_utc().timestamp() as u64,
                parent_slot: self.parent_slot as u64,
                transactions: vec![],
            },
            self.blockchain,
        )
    }
}
