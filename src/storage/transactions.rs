use diesel::prelude::*;
use solana_sdk::{
    account::ReadableAccount,
    transaction::{Legacy, TransactionVersion},
};
use uuid::Uuid;

use crate::engine::transactions::TransactionMetadata;

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
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

impl DbTransaction {
    pub fn from_transaction(blockchain: Uuid, meta: &TransactionMetadata) -> Self {
        DbTransaction {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_utc(),
            signature: meta.tx.signature().to_string(),
            version: version_to_string(&meta.tx.to_versioned_transaction().version()),
            recent_blockhash: meta.tx.message().recent_blockhash().to_bytes().to_vec(),
            slot: meta.current_block.block_height as i64,
            blockchain,
        }
    }
}

pub fn version_to_string(version: &TransactionVersion) -> String {
    match version {
        TransactionVersion::Legacy(_) => "legacy".to_string(),
        TransactionVersion::Number(v) => format!("v{}", v),
    }
}

pub fn string_to_version(version: &str) -> TransactionVersion {
    if version == "legacy" {
        TransactionVersion::Legacy(Legacy::Legacy)
    } else {
        let v = version.trim_start_matches('v').parse().unwrap();
        TransactionVersion::Number(v)
    }
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
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

impl DbTransactionAccountKey {
    pub fn from_transaction(meta: &TransactionMetadata) -> Vec<Self> {
        meta.tx
            .message()
            .account_keys()
            .iter()
            .enumerate()
            .map(|(i, account)| DbTransactionAccountKey {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_utc(),
                transaction_signature: meta.tx.signature().to_string(),
                account: account.to_string(),
                signer: meta.tx.message().is_signer(i),
                writable: meta.tx.message().is_writable(i),
                index: i as i16,
            })
            .collect()
    }
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
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

impl DbTransactionInstruction {
    pub fn from_transaction(meta: &TransactionMetadata) -> Vec<Self> {
        meta.tx
            .message()
            .program_instructions_iter()
            //TODO: I had to imporvise some things, so they may not be perfect
            .map(|(program_id, instruction)| DbTransactionInstruction {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_utc(),
                transaction_signature: meta.tx.signature().to_string(),
                accounts: instruction.accounts.iter().map(|a| *a as i64).collect(),
                data: instruction.data.clone(),
                program_id: program_id.to_bytes().to_vec(),
                stack_height: 1,
                inner: false,
            })
            .collect()
    }

    pub fn to_instruction(
        &self,
        keys: Vec<DbTransactionAccountKey>,
    ) -> solana_sdk::instruction::Instruction {
        let accounts = self
            .accounts
            .iter()
            .map(|a| {
                let key = &keys[*a as usize];
                solana_sdk::instruction::AccountMeta {
                    pubkey: solana_sdk::pubkey::Pubkey::new_from_array(
                        key.account.as_bytes().try_into().unwrap(),
                    ),
                    is_signer: key.signer,
                    is_writable: key.writable,
                }
            })
            .collect();
        let program_id =
            solana_sdk::pubkey::Pubkey::new_from_array(self.program_id.clone().try_into().unwrap());
        let instruction = solana_sdk::instruction::Instruction {
            program_id,
            accounts,
            data: self.data.clone(),
        };
        instruction
    }
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_log_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionLogMessage {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub log: String,
    pub index: i16,
}

impl DbTransactionLogMessage {
    pub fn from_transaction(meta: &TransactionMetadata) -> Vec<Self> {
        meta.logs
            .iter()
            .enumerate()
            .map(|(i, log)| DbTransactionLogMessage {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_utc(),
                transaction_signature: meta.tx.signature().to_string(),
                log: log.to_string(),
                index: i as i16,
            })
            .collect()
    }
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
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

impl DbTransactionMeta {
    pub fn from_transaction(meta: &TransactionMetadata) -> Self {
        DbTransactionMeta {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_utc(),
            transaction_signature: meta.tx.signature().to_string(),
            err: meta.err.as_ref().map(|e| e.to_string()),
            compute_units_consumed: meta.compute_units_consumed as i64,
            fee: meta.tx.message().recent_blockhash().to_bytes()[0] as i64,
            pre_balances: meta
                .pre_accounts
                .iter()
                .map(|(_, a)| a.lamports() as i64)
                .collect(),
            post_balances: meta
                .post_accounts
                .iter()
                .map(|(_, a)| a.lamports() as i64)
                .collect(),
        }
    }
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transaction_signatures)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionSignature {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub signature: String,
}

impl DbTransactionSignature {
    pub fn from_transaction(meta: &TransactionMetadata) -> Vec<Self> {
        meta.tx
            .signatures()
            .iter()
            .map(|signature| DbTransactionSignature {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_utc(),
                transaction_signature: meta.tx.signature().to_string(),
                signature: signature.to_string(),
            })
            .collect()
    }
}
