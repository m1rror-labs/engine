use crate::engine::transactions::TransactionMeta;
use crate::engine::transactions::TransactionMetadata;
use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use diesel::prelude::*;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{
    account::ReadableAccount,
    transaction::{Legacy, TransactionVersion},
};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone, Debug)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransaction {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub signature: String,
    pub version: String,
    pub recent_blockhash: Vec<u8>,
    pub slot: BigDecimal,
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
            slot: meta.current_block.block_height.into(),
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

#[derive(
    Queryable,
    QueryableByName,
    Selectable,
    Insertable,
    AsChangeset,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
)]
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

#[derive(
    Queryable,
    QueryableByName,
    Selectable,
    Insertable,
    AsChangeset,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
)]
#[diesel(table_name = crate::schema::transaction_instructions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionInstruction {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub accounts: Vec<i16>,
    pub data: Vec<u8>,
    pub program_id: String,
    pub stack_height: i16,
    pub inner: bool,
}

impl DbTransactionInstruction {
    pub fn from_transaction(meta: &TransactionMetadata) -> Vec<Self> {
        meta.tx
            .message()
            .program_instructions_iter()
            //TODO: I had to imporvise some things, so they may not be perfect
            .map(|(program_id, instruction)| {
                let mut accounts: Vec<i16> =
                    instruction.accounts.iter().map(|a| *a as i16).collect();
                accounts.push(instruction.program_id_index as i16);
                DbTransactionInstruction {
                    id: Uuid::new_v4(),
                    created_at: chrono::Utc::now().naive_utc(),
                    transaction_signature: meta.tx.signature().to_string(),
                    accounts: instruction.accounts.iter().map(|a| *a as i16).collect(),
                    data: instruction.data.clone(),
                    program_id: program_id.to_string(),
                    stack_height: 1,
                    inner: false,
                }
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
                    pubkey: Pubkey::from_str(&key.account).unwrap(),
                    is_signer: key.signer,
                    is_writable: key.writable,
                }
            })
            .collect();
        let program_id = Pubkey::from_str(&self.program_id).expect("Failed to parse program id");
        let instruction = solana_sdk::instruction::Instruction {
            program_id,
            accounts,
            data: self.data.clone(),
        };
        instruction
    }
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone, Debug)]
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

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone, Debug)]
#[diesel(table_name = crate::schema::transaction_meta)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbTransactionMeta {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub transaction_signature: String,
    pub err: Option<String>,
    pub compute_units_consumed: BigDecimal,
    pub fee: BigDecimal,
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
            compute_units_consumed: meta.compute_units_consumed.into(),
            fee: meta.tx.message().recent_blockhash().to_bytes()[0].into(),
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

    pub fn to_metadata(&self, logs: Vec<DbTransactionLogMessage>) -> TransactionMeta {
        let status = match &self.err {
            Some(_) => serde_json::json!({
                "Err": self.err,
            }),
            None => serde_json::json!({
                "Ok": null,
            }),
        };

        TransactionMeta {
            err: self.err.clone(),
            fee: self.fee.to_u64().unwrap(),
            log_messages: logs.iter().map(|l| l.log.clone()).collect(),
            inner_instructions: Default::default(),
            compute_units_consumed: self.compute_units_consumed.to_u64().unwrap(),
            pre_balances: self
                .pre_balances
                .iter()
                .map(|a| (*a as u64).into())
                .collect(),
            pre_token_balances: vec![],
            post_balances: self
                .post_balances
                .iter()
                .map(|a| (*a as u64).into())
                .collect(),
            post_token_balances: vec![],
            rewards: vec![],
            status: status,
        }
    }
}

#[derive(
    Queryable,
    QueryableByName,
    Selectable,
    Insertable,
    AsChangeset,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
)]
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
