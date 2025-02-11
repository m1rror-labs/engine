use std::collections::HashMap;

use accounts::DbAccount;
use blocks::{DbBlock, DbBlockchain};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::upsert::excluded;

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

    fn set_block(&self, id: Uuid, block: Block) -> Result<(), String>;
    fn get_block(&self, id: Uuid, blockhash: &Hash) -> Result<Block, String>;
    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Option<Block>, String>;
    fn get_latest_block(&self, id: Uuid) -> Result<Block, String>;

    fn get_blockchain(&self, id: Uuid) -> Result<Blockchain, String>;
    fn set_blockchain(&self, blockchain: &Blockchain) -> Result<Uuid, String>;

    fn save_transaction(&self, id: Uuid, tx: &TransactionMetadata) -> Result<(), String>;
    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<Option<Transaction>, String>;
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
        .set(crate::schema::accounts::lamports.eq(lamports as i64))
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
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_block(&self, id: Uuid, block: Block) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        diesel::insert_into(crate::schema::blocks::table)
            .values(DbBlock::from_block(&block, id))
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

        todo!()
    }

    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<Option<Transaction>, String> {
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

        let (db_tx, account_keys, instructions, log_messages, metas, signatures) =
            transaction_map.into_iter().next().unwrap().1;

        Ok(None)
    }
}
