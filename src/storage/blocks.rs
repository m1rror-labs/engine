use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::blockchain)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbBlockchain {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub airdrop_keypair: Vec<u8>,
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
