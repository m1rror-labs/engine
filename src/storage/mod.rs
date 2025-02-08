use accounts::DbAccount;
use blocks::DbBlock;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::upsert::excluded;

use solana_sdk::{
    account::Account, hash::Hash, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use uuid::Uuid;

pub mod accounts;
pub mod blocks;
pub mod transactions;

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
    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Block, String>;
    fn get_latest_block(&self, id: Uuid) -> Result<Block, String>;

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

    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Block, String> {
        let mut conn = self.get_connection()?;
        let block: DbBlock = crate::schema::blocks::table
            .filter(crate::schema::blocks::block_height.eq(height as i64))
            .filter(crate::schema::blocks::blockchain.eq(id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(block.into_block().0)
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
        todo!()
    }

    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<Option<Transaction>, String> {
        todo!()
    }
}
