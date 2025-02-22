use std::collections::HashMap;
use std::str::FromStr;

use accounts::DbAccount;
use bigdecimal::BigDecimal;
use blocks::{DbBlock, DbBlockchain};
use chrono::Utc;
use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types::{Binary, Bool};
use diesel::upsert::excluded;

use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::TransactionError;
use solana_sdk::{
    account::Account, hash::Hash, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use transactions::{
    DbTransaction, DbTransactionAccountKey, DbTransactionInstruction, DbTransactionLogMessage,
    DbTransactionMeta, DbTransactionSignature,
};
use uuid::Uuid;

pub mod accounts;
pub mod blocks;
pub mod transactions;

use crate::engine::blocks::Blockchain;
use crate::engine::{blocks::Block, transactions::TransactionMetadata};

pub trait Storage {
    fn get_account(&self, id: Uuid, address: &Pubkey) -> Result<Option<Account>, String>;
    fn get_accounts(
        &self,
        id: Uuid,
        addresses: &Vec<&Pubkey>,
    ) -> Result<Vec<Option<Account>>, String>;
    fn set_account(
        &self,
        id: Uuid,
        address: &Pubkey,
        account: Account,
        label: Option<String>,
    ) -> Result<(), String>;
    fn set_account_lamports(&self, id: Uuid, address: &Pubkey, lamports: u64)
        -> Result<(), String>;
    fn set_accounts(&self, id: Uuid, accounts: Vec<(Pubkey, Account)>) -> Result<(), String>;
    fn get_token_accounts_by_owner(
        &self,
        id: Uuid,
        owner: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String>;

    fn set_block(&self, id: Uuid, block: &Block) -> Result<(), String>;
    fn get_block(&self, id: Uuid, blockhash: &Hash) -> Result<Block, String>;
    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Option<Block>, String>;
    fn get_block_created_at(&self, id: Uuid, height: u64) -> Result<chrono::DateTime<Utc>, String>;
    fn get_latest_block(&self, id: Uuid) -> Result<Block, String>;

    fn get_blockchain(&self, id: Uuid) -> Result<Blockchain, String>;
    fn get_blockchains(&self) -> Result<Vec<Blockchain>, String>;
    fn delete_blockchain(&self, id: Uuid) -> Result<(), String>;
    fn set_blockchain(&self, blockchain: &Blockchain) -> Result<Uuid, String>;

    fn save_transaction(&self, id: Uuid, tx: &TransactionMetadata) -> Result<(), String>;
    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<
        Option<(
            Transaction,
            u64,
            Option<TransactionError>,
            chrono::NaiveDateTime,
        )>,
        String,
    >;
    fn get_transactions_for_address(
        &self,
        id: Uuid,
        address: &Pubkey,
        limit: Option<usize>,
    ) -> Result<Vec<DbTransaction>, String>;
    fn get_transaction_count(&self, id: Uuid) -> Result<u64, String>;
}

type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct PgStorage {
    pool: PgPool,
}

impl PgStorage {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        PgStorage { pool }
    }

    fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, String> {
        self.pool.get().map_err(|e| e.to_string())
    }
}

impl Storage for PgStorage {
    fn get_blockchain(&self, id: Uuid) -> Result<Blockchain, String> {
        let mut conn = self.get_connection()?;
        let blockchain = crate::schema::blockchain::table
            .filter(crate::schema::blockchain::id.eq(id))
            .first::<DbBlockchain>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(blockchain.to_blockchain())
    }
    fn get_blockchains(&self) -> Result<Vec<Blockchain>, String> {
        let mut conn = self.get_connection()?;
        let blockchains = crate::schema::blockchain::table
            .load::<DbBlockchain>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(blockchains.into_iter().map(|b| b.to_blockchain()).collect())
    }

    fn set_blockchain(&self, blockchain: &Blockchain) -> Result<Uuid, String> {
        let mut conn = self.get_connection()?;
        let db_blockchain = DbBlockchain {
            id: blockchain.id,
            created_at: blockchain.created_at,
            airdrop_keypair: blockchain.airdrop_keypair.to_bytes().to_vec(),
        };
        diesel::insert_into(crate::schema::blockchain::table)
            .values(&db_blockchain)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(blockchain.id)
    }

    fn delete_blockchain(&self, id: Uuid) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        diesel::delete(
            crate::schema::blockchain::table.filter(crate::schema::blockchain::id.eq(id)),
        )
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_account(&self, id: Uuid, address: &Pubkey) -> Result<Option<Account>, String> {
        let mut conn = self.get_connection()?;
        let account = crate::schema::accounts::table
            .filter(crate::schema::accounts::address.eq(address.to_string()))
            .filter(crate::schema::accounts::blockchain.eq(id))
            .first::<DbAccount>(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(account.map(|a| a.into_account()))
    }

    fn get_accounts(
        &self,
        id: Uuid,
        addresses: &Vec<&Pubkey>,
    ) -> Result<Vec<Option<Account>>, String> {
        let mut conn = self.get_connection()?;
        let accounts = crate::schema::accounts::table
            .filter(
                crate::schema::accounts::address.eq_any(addresses.iter().map(|a| a.to_string())),
            )
            .filter(crate::schema::accounts::blockchain.eq(id))
            .load::<DbAccount>(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(addresses
            .iter()
            .map(|address| {
                accounts
                    .iter()
                    .find(|a| a.address == address.to_string())
                    .map(|a| a.clone().into_account())
            })
            .collect())
    }

    fn set_account_lamports(
        &self,
        id: Uuid,
        address: &Pubkey,
        lamports: u64,
    ) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        diesel::update(
            crate::schema::accounts::table
                .filter(crate::schema::accounts::address.eq(address.to_string()))
                .filter(crate::schema::accounts::blockchain.eq(id)),
        )
        .set(crate::schema::accounts::lamports.eq::<BigDecimal>(lamports.into()))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_account(
        &self,
        id: Uuid,
        address: &Pubkey,
        account: Account,
        label: Option<String>,
    ) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        let db_account = DbAccount::from_account(address, &account, label, id);
        diesel::insert_into(crate::schema::accounts::table)
            .values(&db_account)
            .on_conflict((
                crate::schema::accounts::address,
                crate::schema::accounts::blockchain,
            ))
            .do_update()
            .set((
                crate::schema::accounts::lamports.eq(excluded(crate::schema::accounts::lamports)),
                crate::schema::accounts::data.eq(excluded(crate::schema::accounts::data)),
                crate::schema::accounts::owner.eq(excluded(crate::schema::accounts::owner)),
                crate::schema::accounts::executable
                    .eq(excluded(crate::schema::accounts::executable)),
                crate::schema::accounts::rent_epoch
                    .eq(excluded(crate::schema::accounts::rent_epoch)),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_accounts(&self, id: Uuid, accounts: Vec<(Pubkey, Account)>) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        let db_accounts: Vec<DbAccount> = accounts
            .iter()
            .map(|(address, account)| DbAccount::from_account(address, account, None, id))
            .collect();
        diesel::insert_into(crate::schema::accounts::table)
            .values(db_accounts)
            .on_conflict((
                crate::schema::accounts::address,
                crate::schema::accounts::blockchain,
            ))
            .do_update()
            .set((
                crate::schema::accounts::lamports.eq(excluded(crate::schema::accounts::lamports)),
                crate::schema::accounts::data.eq(excluded(crate::schema::accounts::data)),
                crate::schema::accounts::owner.eq(excluded(crate::schema::accounts::owner)),
                crate::schema::accounts::executable
                    .eq(excluded(crate::schema::accounts::executable)),
                crate::schema::accounts::rent_epoch
                    .eq(excluded(crate::schema::accounts::rent_epoch)),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_token_accounts_by_owner(
        &self,
        id: Uuid,
        owner: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String> {
        let mut conn = self.get_connection()?;
        let accounts = crate::schema::accounts::table
            .filter(crate::schema::accounts::owner.eq(token_program.to_string()))
            .filter(sql::<Bool>("contains(data, ?)").bind::<Binary, _>(owner.to_bytes().to_vec()))
            .filter(crate::schema::accounts::blockchain.eq(id))
            .load::<DbAccount>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(accounts
            .iter()
            .map(|a| {
                (
                    Pubkey::from_str(&a.address).unwrap(),
                    a.clone().into_account(),
                )
            })
            .collect())
    }

    fn set_block(&self, id: Uuid, block: &Block) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        diesel::insert_into(crate::schema::blocks::table)
            .values(DbBlock::from_block(block, id))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_block(&self, id: Uuid, blockhash: &Hash) -> Result<Block, String> {
        let mut conn = self.get_connection()?;
        let block: DbBlock = crate::schema::blocks::table
            .filter(crate::schema::blocks::blockhash.eq(blockhash.to_bytes().to_vec()))
            .filter(crate::schema::blocks::blockchain.eq(id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(block.into_block().0)
    }

    //TODO: Need to do a join on transactions to get the transactions for the block
    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Option<Block>, String> {
        let mut conn = self.get_connection()?;
        let block: Option<DbBlock> = crate::schema::blocks::table
            .filter(crate::schema::blocks::block_height.eq(height as i64))
            .filter(crate::schema::blocks::blockchain.eq(id))
            .first(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;
        match block {
            Some(block) => Ok(Some(block.into_block().0)),
            None => Ok(None),
        }
    }

    fn get_block_created_at(&self, id: Uuid, height: u64) -> Result<chrono::DateTime<Utc>, String> {
        let mut conn = self.get_connection()?;
        let block: DbBlock = crate::schema::blocks::table
            .filter(crate::schema::blocks::block_height.eq(height as i64))
            .filter(crate::schema::blocks::blockchain.eq(id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(block.created_at.and_utc())
    }

    fn get_latest_block(&self, id: Uuid) -> Result<Block, String> {
        let mut conn = self.get_connection()?;
        let block: DbBlock = crate::schema::blocks::table
            .filter(crate::schema::blocks::blockchain.eq(id))
            .order(crate::schema::blocks::block_height.desc())
            .first(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(block.into_block().0)
    }

    fn save_transaction(&self, id: Uuid, tx: &TransactionMetadata) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        // save transaction
        diesel::insert_into(crate::schema::transactions::table)
            .values(DbTransaction::from_transaction(id, tx))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        // save transaction account keys
        diesel::insert_into(crate::schema::transaction_account_keys::table)
            .values(DbTransactionAccountKey::from_transaction(tx))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        // save transaction instructions
        diesel::insert_into(crate::schema::transaction_instructions::table)
            .values(DbTransactionInstruction::from_transaction(tx))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        // save transaction log messages
        diesel::insert_into(crate::schema::transaction_log_messages::table)
            .values(DbTransactionLogMessage::from_transaction(tx))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        // save transaction meta
        diesel::insert_into(crate::schema::transaction_meta::table)
            .values(DbTransactionMeta::from_transaction(tx))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        // save transaction signatures
        diesel::insert_into(crate::schema::transaction_signatures::table)
            .values(DbTransactionSignature::from_transaction(tx))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<
        Option<(
            Transaction,
            u64,
            Option<TransactionError>,
            chrono::NaiveDateTime,
        )>,
        String,
    > {
        let mut conn = self.get_connection()?;

        let res: Vec<(
            DbTransaction,
            Option<DbTransactionAccountKey>,
            Option<DbTransactionInstruction>,
            Option<DbTransactionLogMessage>,
            Option<DbTransactionMeta>,
            Option<DbTransactionSignature>,
        )> = crate::schema::transactions::table
            .left_join(
                crate::schema::transaction_account_keys::table
                    .on(crate::schema::transactions::signature
                        .eq(crate::schema::transaction_account_keys::transaction_signature)),
            )
            .left_join(
                crate::schema::transaction_instructions::table
                    .on(crate::schema::transactions::signature
                        .eq(crate::schema::transaction_instructions::transaction_signature)),
            )
            .left_join(
                crate::schema::transaction_log_messages::table
                    .on(crate::schema::transactions::signature
                        .eq(crate::schema::transaction_log_messages::transaction_signature)),
            )
            .left_join(
                crate::schema::transaction_meta::table.on(crate::schema::transactions::signature
                    .eq(crate::schema::transaction_meta::transaction_signature)),
            )
            .left_join(
                crate::schema::transaction_signatures::table
                    .on(crate::schema::transactions::signature
                        .eq(crate::schema::transaction_signatures::transaction_signature)),
            )
            .filter(crate::schema::transactions::signature.eq(signature.to_string()))
            .filter(crate::schema::transactions::blockchain.eq(id))
            .load::<(
                DbTransaction,
                Option<DbTransactionAccountKey>,
                Option<DbTransactionInstruction>,
                Option<DbTransactionLogMessage>,
                Option<DbTransactionMeta>,
                Option<DbTransactionSignature>,
            )>(&mut conn)
            .map_err(|e| e.to_string())?;

        let mut transaction_map: HashMap<
            Uuid,
            (
                DbTransaction,
                Vec<DbTransactionAccountKey>,
                Vec<DbTransactionInstruction>,
                Vec<DbTransactionLogMessage>,
                Vec<DbTransactionMeta>,
                Vec<DbTransactionSignature>,
            ),
        > = HashMap::new();

        for (tx, account_key, instruction, log_message, meta, signature) in res {
            let entry = transaction_map.entry(tx.id).or_insert((
                tx,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));
            if let Some(account_key) = account_key {
                entry.1.push(account_key);
            };
            if let Some(instruction) = instruction {
                entry.2.push(instruction);
            };
            if let Some(log_message) = log_message {
                entry.3.push(log_message);
            };
            if let Some(meta) = meta {
                entry.4.push(meta);
            };
            if let Some(signature) = signature {
                entry.5.push(signature);
            };
        }

        if transaction_map.is_empty() {
            return Ok(None);
        }

        if transaction_map.len() > 1 {
            return Err("Multiple transactions found with the same signature".to_string());
        }

        let (db_tx, account_keys, instructions, _, metas, signatures) =
            transaction_map.into_iter().next().unwrap().1;

        let instructions = instructions
            .iter()
            .map(|i| i.to_instruction(account_keys.clone()))
            .collect::<Vec<Instruction>>();

        let tx_meta = metas.first().ok_or_else(|| "No meta found".to_string())?;

        Ok(Some((
            Transaction {
                signatures: signatures
                    .into_iter()
                    .map(|s| Signature::from_str(&s.signature).unwrap())
                    .collect(),
                message: solana_sdk::message::Message::new(&instructions, None),
            },
            db_tx.slot as u64,
            match tx_meta.to_owned().err {
                Some(e) => {
                    let deserialized_error: Result<TransactionError, _> = serde_json::from_str(&e);
                    match deserialized_error {
                        Ok(e) => Some(e),
                        Err(_) => Some(TransactionError::InvalidAccountIndex),
                    }
                }
                None => None,
            },
            db_tx.created_at,
        )))
    }

    fn get_transactions_for_address(
        &self,
        id: Uuid,
        address: &Pubkey,
        limit: Option<usize>,
    ) -> Result<Vec<DbTransaction>, String> {
        let mut conn = self.get_connection()?;
        let transactions: Vec<DbTransaction> = crate::schema::transactions::table
            .inner_join(
                crate::schema::transaction_account_keys::table
                    .on(crate::schema::transactions::signature
                        .eq(crate::schema::transaction_account_keys::transaction_signature)),
            )
            .filter(crate::schema::transaction_account_keys::account.eq(address.to_string()))
            .filter(crate::schema::transactions::blockchain.eq(id))
            .select(crate::schema::transactions::all_columns)
            .limit(limit.unwrap_or(1000) as i64)
            .load(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(transactions)
    }

    fn get_transaction_count(&self, id: Uuid) -> Result<u64, String> {
        let mut conn = self.get_connection()?;
        let count: i64 = crate::schema::transactions::table
            .filter(crate::schema::transactions::blockchain.eq(id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(count as u64)
    }
}
