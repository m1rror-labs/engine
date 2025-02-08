use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransaction {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub signature: String,
    pub version: String,
    pub recent_blockhash: Vec<u8>,
    pub slot: i64,
    pub blockchain: Uuid,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_account_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionAccountKey {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub account: String,
    pub signer: bool,
    pub writable: bool,
    pub index: i16,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_instructions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionInstruction {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub accounts: Vec<i64>,
    pub data: Vec<u8>,
    pub program_id: Vec<u8>,
    pub stack_height: i16,
    pub inner: bool,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_log_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionLogMessage {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub log: String,
    pub index: i16,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_meta)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionMeta {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub err: Option<String>,
    pub compute_units_consumed: i64,
    pub fee: i64,
    pub pre_balances: Vec<i64>,
    pub post_balances: Vec<i64>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_signatures)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionSignature {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub signature: String,
}
