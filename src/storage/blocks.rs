use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::prelude::*;
use solana_sdk::{hash::Hash, signature::Keypair};
use uuid::Uuid;

use crate::engine::blocks::{Block, Blockchain};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::blockchains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbBlockchain {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub airdrop_keypair: Vec<u8>,
    pub team_id: Uuid,
    pub label: Option<String>,
    pub expiry: Option<chrono::NaiveDateTime>,
}

impl DbBlockchain {
    pub fn to_blockchain(self) -> Blockchain {
        Blockchain {
            id: self.id,
            created_at: self.created_at,
            airdrop_keypair: Keypair::from_bytes(self.airdrop_keypair.as_slice()).unwrap(),
            team_id: self.team_id,
            label: self.label,
            expiry: self.expiry,
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
    pub parent_slot: BigDecimal,
    pub block_height: BigDecimal,
    pub slot: BigDecimal,
}

impl DbBlock {
    pub fn from_block(block: &Block, blockchain: Uuid) -> Self {
        DbBlock {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_utc(),
            blockchain,
            blockhash: block.blockhash.to_bytes().to_vec(),
            previous_blockhash: block.previous_blockhash.to_bytes().to_vec(),
            parent_slot: block.parent_slot.into(),
            block_height: block.block_height.into(),
            slot: block.block_height.into(),
        }
    }

    pub fn into_block(self) -> (Block, Uuid) {
        (
            Block {
                blockhash: Hash::new_from_array(self.blockhash.as_slice().try_into().unwrap()),
                previous_blockhash: Hash::new_from_array(
                    self.previous_blockhash.as_slice().try_into().unwrap(),
                ),
                block_height: self.block_height.to_u64().unwrap(),
                block_time: self.created_at.and_utc().timestamp() as u64,
                parent_slot: self.parent_slot.to_u64().unwrap(),
                transactions: vec![],
            },
            self.blockchain,
        )
    }
}
