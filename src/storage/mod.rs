use accounts::{DbAccount, DbConfigAccount};
use actix_web::rt;
use bigdecimal::{BigDecimal, ToPrimitive};
use blocks::{DbBlock, DbBlockchain};
use cache::Cache;
use chrono::Utc;
use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types::{Bool, Text};
use diesel::upsert::excluded;
use hex::encode;
use pubsub::Pubsub;
use rpc::Rpc;
use std::str::FromStr;

use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::TransactionError;
use solana_sdk::{
    account::Account, hash::Hash, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use teams::Team;
use transactions::{
    DBTransactionTokenBalance, DbTransaction, DbTransactionAccountKey, DbTransactionInstruction,
    DbTransactionLogMessage, DbTransactionMeta, DbTransactionObject, DbTransactionSignature,
};
use uuid::Uuid;

pub mod accounts;
pub mod blocks;
pub mod cache;
pub mod pubsub;
pub mod rpc;
pub mod teams;
pub mod transactions;

use crate::engine::blocks::Blockchain;
use crate::engine::transactions::TransactionMeta;
use crate::engine::{blocks::Block, transactions::TransactionMetadata};

pub trait Storage {
    fn get_team_from_api_key(&self, api_key: Uuid) -> Result<Team, String>;

    fn get_account(&self, id: Uuid, address: &Pubkey) -> Result<Option<Account>, String>;
    fn get_account_jit(
        &self,
        id: Uuid,
        address: &Pubkey,
        jit: bool,
    ) -> impl std::future::Future<Output = Result<Option<Account>, String>> + Send;
    fn get_accounts(
        &self,
        id: Uuid,
        addresses: &Vec<&Pubkey>,
    ) -> Result<Vec<Option<Account>>, String>;
    fn get_accounts_jit(
        &self,
        id: Uuid,
        addresses: &Vec<&Pubkey>,
        jit: bool,
    ) -> impl std::future::Future<Output = Result<Vec<Option<Account>>, String>> + Send;
    fn get_largest_accounts(&self, id: Uuid, limit: usize) -> Result<Vec<(Pubkey, u64)>, String>;
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
    fn get_program_accounts(
        &self,
        id: Uuid,
        program_id: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String>;
    fn get_config_accounts(&self, config_id: Uuid) -> Result<Vec<(Pubkey, Account)>, String>;
    fn get_config_account(
        &self,
        config_id: Uuid,
        pubkey: &Pubkey,
    ) -> Result<Option<Account>, String>;
    fn set_config_account(
        &self,
        config_id: Uuid,
        address: &Pubkey,
        account: Account,
    ) -> Result<(), String>;

    fn set_block(&self, id: Uuid, block: &Block) -> Result<(), String>;
    fn get_block(&self, id: Uuid, blockhash: &Hash) -> Result<Block, String>;
    fn get_recent_blocks(&self, id: Uuid, limit: usize) -> Result<Vec<Block>, String>;
    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Option<Block>, String>;
    fn get_block_created_at(&self, id: Uuid, height: u64) -> Result<chrono::DateTime<Utc>, String>;
    fn get_latest_block(&self, id: Uuid) -> Result<Block, String>;

    fn get_blockchain(&self, id: Uuid) -> Result<Blockchain, String>;
    fn get_expired_blockchains(&self) -> Result<Vec<Blockchain>, String>;
    fn get_blockchains(&self, team_id: Uuid) -> Result<Vec<Blockchain>, String>;
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
            TransactionMeta,
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
    fn get_transactions_for_address_created_at(
        &self,
        id: Uuid,
        address: &Pubkey,
        start: chrono::NaiveDateTime,
        end: chrono::NaiveDateTime,
    ) -> Result<Vec<DbTransaction>, String>;
    fn get_transaction_count(&self, id: Uuid) -> Result<u64, String>;
}

type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct PgStorage {
    pool: PgPool,
    cache: Cache,
    rpc: Rpc,
    pubsub: Pubsub,
}

impl PgStorage {
    pub fn new(database_url: &str, cache_url: &str, rpc_url: &str, pubsub_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = match r2d2::Pool::builder().build(manager) {
            Ok(pool) => pool,
            Err(e) => panic!("Failed to create pool: {}", e),
        };

        PgStorage {
            pool,
            cache: Cache::new(cache_url),
            rpc: Rpc::new(rpc_url.to_string()),
            pubsub: Pubsub::new(pubsub_url),
        }
    }

    fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, String> {
        self.pool.get().map_err(|e| e.to_string())
    }
}

impl Storage for PgStorage {
    fn get_team_from_api_key(&self, api_key: Uuid) -> Result<Team, String> {
        let mut conn = self.get_connection()?;
        let team = crate::schema::api_keys::table
            .filter(crate::schema::api_keys::id.eq(api_key))
            .inner_join(
                crate::schema::teams::table
                    .on(crate::schema::api_keys::team_id.eq(crate::schema::teams::id)),
            )
            .select(crate::schema::teams::all_columns)
            .first::<Team>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(team)
    }

    fn get_blockchain(&self, id: Uuid) -> Result<Blockchain, String> {
        let mut conn = self.get_connection()?;
        let blockchain = crate::schema::blockchains::table
            .filter(crate::schema::blockchains::id.eq(id))
            .first::<DbBlockchain>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(blockchain.to_blockchain())
    }
    fn get_expired_blockchains(&self) -> Result<Vec<Blockchain>, String> {
        let mut conn = self.get_connection()?;
        let blockchains = crate::schema::blockchains::table
            .filter(crate::schema::blockchains::expiry.lt(chrono::Utc::now().naive_utc()))
            .load::<DbBlockchain>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(blockchains.into_iter().map(|b| b.to_blockchain()).collect())
    }
    fn get_blockchains(&self, team_id: Uuid) -> Result<Vec<Blockchain>, String> {
        let mut conn = self.get_connection()?;
        let blockchains = crate::schema::blockchains::table
            .filter(crate::schema::blockchains::team_id.eq(team_id))
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
            team_id: blockchain.team_id,
            label: blockchain.label.clone(),
            expiry: blockchain.expiry,
            jit: blockchain.jit,
        };
        diesel::insert_into(crate::schema::blockchains::table)
            .values(&db_blockchain)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(blockchain.id)
    }

    fn delete_blockchain(&self, id: Uuid) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        diesel::delete(
            crate::schema::blockchains::table.filter(crate::schema::blockchains::id.eq(id)),
        )
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_account(&self, id: Uuid, address: &Pubkey) -> Result<Option<Account>, String> {
        let account = self.cache.get_account(id, &address.to_string())?;
        Ok(account.map(|a| a.into_account()))
    }

    async fn get_account_jit(
        &self,
        id: Uuid,
        address: &Pubkey,
        jit: bool,
    ) -> Result<Option<Account>, String> {
        let account = self.cache.get_account(id, &address.to_string())?;
        if account.is_none() && jit {
            let mainnet_account = self.rpc.get_account(address).await?;
            if mainnet_account.is_some() {
                self.set_account(id, address, mainnet_account.clone().unwrap(), None)?;
            }
            return Ok(mainnet_account);
        }

        Ok(account.map(|a| a.into_account()))
    }

    fn get_accounts(
        &self,
        id: Uuid,
        addresses: &Vec<&Pubkey>,
    ) -> Result<Vec<Option<Account>>, String> {
        let accounts = self.cache.get_accounts(
            id,
            addresses
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>(),
        )?;

        Ok(accounts
            .iter()
            .map(|a| a.as_ref().map(|a| a.clone().into_account()))
            .collect())
    }

    async fn get_accounts_jit(
        &self,
        id: Uuid,
        addresses: &Vec<&Pubkey>,
        jit: bool,
    ) -> Result<Vec<Option<Account>>, String> {
        let mut accounts = self.cache.get_accounts(
            id,
            addresses
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>(),
        )?;
        if jit {
            let none_accounts = accounts
                .iter()
                .enumerate()
                .filter(|(_, a)| a.is_none())
                .map(|(idx, _)| addresses[idx].to_owned())
                .collect::<Vec<Pubkey>>();

            let none_idxs = accounts
                .iter()
                .enumerate()
                .filter(|(_, a)| a.is_none())
                .map(|(idx, _)| idx)
                .collect::<Vec<usize>>();

            let mainnet_accounts = self.rpc.get_accounts(&none_accounts).await?;
            let mut accounts_to_save = vec![];
            for (i, account) in mainnet_accounts.iter().enumerate() {
                let idx = none_idxs[i];
                if let Some(account) = account {
                    accounts_to_save.push((addresses[idx].to_owned(), account.clone()));
                    accounts.insert(
                        none_idxs[idx],
                        Some(DbAccount::from_account(addresses[idx], &account, None, id)),
                    );
                }
            }
            if accounts_to_save.len() > 0 {
                self.set_accounts(id, accounts_to_save)?;
            }
        }

        Ok(accounts
            .iter()
            .map(|a| a.as_ref().map(|a| a.clone().into_account()))
            .collect())
    }
    fn get_largest_accounts(&self, id: Uuid, limit: usize) -> Result<Vec<(Pubkey, u64)>, String> {
        let mut conn = self.get_connection()?;
        let accounts = crate::schema::accounts::table
            .filter(crate::schema::accounts::blockchain.eq(id))
            .order(crate::schema::accounts::lamports.desc())
            .limit(limit as i64)
            .load::<DbAccount>(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(accounts
            .iter()
            .map(|a| {
                (
                    Pubkey::from_str(&a.address).unwrap(),
                    a.lamports.to_u64().unwrap(),
                )
            })
            .collect())
    }

    fn set_account_lamports(
        &self,
        id: Uuid,
        address: &Pubkey,
        lamports: u64,
    ) -> Result<(), String> {
        let account = self.cache.get_account(id, &address.to_string())?;
        if let Some(mut account) = account {
            account.lamports = lamports.into();
            self.cache.set_accounts(id, vec![account])?;
        }

        let self_clone = self.clone();
        let address_clone = address.clone();
        rt::spawn(async move {
            let mut conn = self_clone.get_connection().unwrap();
            diesel::update(
                crate::schema::accounts::table
                    .filter(crate::schema::accounts::address.eq(address_clone.to_string()))
                    .filter(crate::schema::accounts::blockchain.eq(id)),
            )
            .set(crate::schema::accounts::lamports.eq::<BigDecimal>(lamports.into()))
            .execute(&mut conn)
            .map_err(|e| e.to_string())
            .unwrap();
        });
        Ok(())
    }

    fn set_account(
        &self,
        id: Uuid,
        address: &Pubkey,
        account: Account,
        label: Option<String>,
    ) -> Result<(), String> {
        let db_account = DbAccount::from_account(&address.clone(), &account, label.clone(), id);
        self.cache.set_accounts(id, vec![db_account.clone()])?;
        self.pubsub.publish_account_update(db_account.clone());

        let self_clone = self.clone();
        let address_clone = address.clone();
        rt::spawn(async move {
            let mut conn = self_clone.get_connection().unwrap();
            let db_account = DbAccount::from_account(&address_clone, &account, label, id);
            diesel::insert_into(crate::schema::accounts::table)
                .values(&db_account)
                .on_conflict((
                    crate::schema::accounts::address,
                    crate::schema::accounts::blockchain,
                ))
                .do_update()
                .set((
                    crate::schema::accounts::lamports
                        .eq(excluded(crate::schema::accounts::lamports)),
                    crate::schema::accounts::data.eq(excluded(crate::schema::accounts::data)),
                    crate::schema::accounts::owner.eq(excluded(crate::schema::accounts::owner)),
                    crate::schema::accounts::executable
                        .eq(excluded(crate::schema::accounts::executable)),
                    crate::schema::accounts::rent_epoch
                        .eq(excluded(crate::schema::accounts::rent_epoch)),
                ))
                .execute(&mut conn)
                .map_err(|e| e.to_string())
                .unwrap();
        });
        Ok(())
    }

    fn set_accounts(&self, id: Uuid, accounts: Vec<(Pubkey, Account)>) -> Result<(), String> {
        let db_accounts: Vec<DbAccount> = accounts
            .iter()
            .map(|(address, account)| DbAccount::from_account(address, account, None, id))
            .collect();
        self.cache.set_accounts(id, db_accounts.clone())?;
        self.pubsub.publish_accounts_update(db_accounts.clone());

        let self_clone = self.clone();
        rt::spawn(async move {
            let mut conn = self_clone.get_connection().unwrap();
            let db_accounts: Vec<DbAccount> = accounts
                .iter()
                .map(|(address, account)| DbAccount::from_account(address, account, None, id))
                .collect();
            conn.transaction::<_, diesel::result::Error, _>(|conn| {
                diesel::insert_into(crate::schema::accounts::table)
                    .values(db_accounts)
                    .on_conflict((
                        crate::schema::accounts::address,
                        crate::schema::accounts::blockchain,
                    ))
                    .do_update()
                    .set((
                        crate::schema::accounts::lamports
                            .eq(excluded(crate::schema::accounts::lamports)),
                        crate::schema::accounts::data.eq(excluded(crate::schema::accounts::data)),
                        crate::schema::accounts::owner.eq(excluded(crate::schema::accounts::owner)),
                        crate::schema::accounts::executable
                            .eq(excluded(crate::schema::accounts::executable)),
                        crate::schema::accounts::rent_epoch
                            .eq(excluded(crate::schema::accounts::rent_epoch)),
                    ))
                    .execute(conn)
            })
            .unwrap();
        });
        Ok(())
    }

    fn get_token_accounts_by_owner(
        &self,
        id: Uuid,
        owner: &Pubkey,
        token_program: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String> {
        let mut conn = self.get_connection()?;
        let owner_hex = encode(owner.to_bytes());
        let query = crate::schema::accounts::table
            .filter(crate::schema::accounts::owner.eq(token_program.to_string()))
            .filter(
                sql::<Bool>("position(decode(")
                    .bind::<Text, _>(owner_hex)
                    .sql(", 'hex') IN data) > 0"),
            )
            .filter(crate::schema::accounts::blockchain.eq(id));

        let accounts = query
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
    fn get_program_accounts(
        &self,
        id: Uuid,
        program_id: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String> {
        let mut conn = self.get_connection()?;
        let accounts = crate::schema::accounts::table
            .filter(crate::schema::accounts::owner.eq(program_id.to_string()))
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
    fn get_config_accounts(&self, config_id: Uuid) -> Result<Vec<(Pubkey, Account)>, String> {
        let mut conn = self.get_connection()?;
        let accounts = crate::schema::blockchain_config_accounts::table
            .filter(crate::schema::blockchain_config_accounts::config.eq(config_id))
            .load::<DbConfigAccount>(&mut conn)
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
    fn get_config_account(
        &self,
        config_id: Uuid,
        pubkey: &Pubkey,
    ) -> Result<Option<Account>, String> {
        let mut conn = self.get_connection()?;
        let account = crate::schema::blockchain_config_accounts::table
            .filter(crate::schema::blockchain_config_accounts::config.eq(config_id))
            .filter(crate::schema::blockchain_config_accounts::address.eq(pubkey.to_string()))
            .first::<DbConfigAccount>(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(account.map(|a| a.into_account()))
    }
    fn set_config_account(
        &self,
        config_id: Uuid,
        address: &Pubkey,
        account: Account,
    ) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        let db_account = DbConfigAccount::from_account(address, &account, None, config_id);
        diesel::insert_into(crate::schema::blockchain_config_accounts::table)
            .values(&db_account)
            .on_conflict((
                crate::schema::blockchain_config_accounts::address,
                crate::schema::blockchain_config_accounts::config,
            ))
            .do_update()
            .set((
                crate::schema::blockchain_config_accounts::lamports.eq(excluded(
                    crate::schema::blockchain_config_accounts::lamports,
                )),
                crate::schema::blockchain_config_accounts::data
                    .eq(excluded(crate::schema::blockchain_config_accounts::data)),
                crate::schema::blockchain_config_accounts::owner
                    .eq(excluded(crate::schema::blockchain_config_accounts::owner)),
                crate::schema::blockchain_config_accounts::executable.eq(excluded(
                    crate::schema::blockchain_config_accounts::executable,
                )),
                crate::schema::blockchain_config_accounts::rent_epoch.eq(excluded(
                    crate::schema::blockchain_config_accounts::rent_epoch,
                )),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_block(&self, id: Uuid, block: &Block) -> Result<(), String> {
        let self_clone = self.clone();
        let db_block = DbBlock::from_block(block, id);
        self.cache.set_block(id, db_block.clone())?;
        self.pubsub.publish_block(db_block.clone());

        rt::spawn(async move {
            let mut conn = self_clone.get_connection().unwrap();
            diesel::insert_into(crate::schema::blocks::table)
                .values(db_block)
                .execute(&mut conn)
                .map_err(|e| e.to_string())
                .unwrap();
        });
        Ok(())
    }

    fn get_block(&self, id: Uuid, blockhash: &Hash) -> Result<Block, String> {
        let block = self.cache.get_block(id, &blockhash.to_bytes())?;
        match block {
            Some(block) => Ok(block.into_block().0),
            None => {
                let mut conn = self.get_connection()?;
                let block: DbBlock = crate::schema::blocks::table
                    .filter(crate::schema::blocks::blockhash.eq(blockhash.to_bytes()))
                    .filter(crate::schema::blocks::blockchain.eq(id))
                    .first(&mut conn)
                    .map_err(|e| e.to_string())?;
                Ok(block.into_block().0)
            }
        }
    }

    fn get_recent_blocks(&self, id: Uuid, limit: usize) -> Result<Vec<Block>, String> {
        let blocks = self.cache.get_recent_blocks(id, limit);
        match blocks {
            Ok(blocks) => Ok(blocks.into_iter().map(|b| b.into_block().0).collect()),
            Err(e) => Err(e),
        }
    }

    //TODO: Need to do a join on transactions to get the transactions for the block
    fn get_block_by_height(&self, id: Uuid, height: u64) -> Result<Option<Block>, String> {
        let mut conn = self.get_connection()?;
        let block: Option<DbBlock> = crate::schema::blocks::table
            .filter(crate::schema::blocks::block_height.eq::<BigDecimal>(height.into()))
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
            .filter(crate::schema::blocks::block_height.eq::<BigDecimal>(height.into()))
            .filter(crate::schema::blocks::blockchain.eq(id))
            .first(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(block.created_at.and_utc())
    }

    fn get_latest_block(&self, id: Uuid) -> Result<Block, String> {
        let block = self.cache.get_latest_block(id)?;
        Ok(block.into_block().0)
    }

    fn save_transaction(&self, id: Uuid, tx: &TransactionMetadata) -> Result<(), String> {
        let mut conn = self.get_connection()?;
        let db_tx = DbTransaction::from_transaction(id, &tx);
        let db_meta = DbTransactionMeta::from_transaction(tx);
        let db_accounts = DbTransactionAccountKey::from_transaction(tx);
        let db_ix = DbTransactionInstruction::from_transaction(tx);
        let db_log = DbTransactionLogMessage::from_transaction(tx);
        let db_signature = DbTransactionSignature::from_transaction(tx);
        let mut token_balances: Vec<DBTransactionTokenBalance> = Vec::new();
        if let Some(pre_balances) = &tx.pre_token_balances {
            for pre_balance in pre_balances {
                token_balances.push(DBTransactionTokenBalance::from_token_balance(
                    pre_balance,
                    &tx.signature.to_string(),
                    true,
                ));
            }
        }
        if let Some(post_balances) = &tx.post_token_balances {
            for post_balance in post_balances {
                token_balances.push(DBTransactionTokenBalance::from_token_balance(
                    post_balance,
                    &tx.signature.to_string(),
                    false,
                ));
            }
        }
        let tx_object = DbTransactionObject {
            transaction: db_tx.clone(),
            meta: db_meta.clone(),
            account_keys: db_accounts.clone(),
            instructions: db_ix.clone(),
            log_messages: db_log.clone(),
            signatures: db_signature.clone(),
            token_balances: token_balances.clone(),
        };
        self.cache.set_transaction(id, tx_object.clone())?;
        self.pubsub.publish_transaction(tx_object.clone());

        rt::spawn(async move {
            diesel::insert_into(crate::schema::transactions::table)
                .values(db_tx)
                .execute(&mut conn)
                .map_err(|e| e.to_string())
                .unwrap();
            diesel::insert_into(crate::schema::transaction_meta::table)
                .values(db_meta)
                .execute(&mut conn)
                .unwrap();
            diesel::insert_into(crate::schema::transaction_account_keys::table)
                .values(db_accounts)
                .execute(&mut conn)
                .unwrap();
            diesel::insert_into(crate::schema::transaction_instructions::table)
                .values(db_ix)
                .execute(&mut conn)
                .unwrap();
            diesel::insert_into(crate::schema::transaction_log_messages::table)
                .values(db_log)
                .execute(&mut conn)
                .unwrap();
            diesel::insert_into(crate::schema::transaction_signatures::table)
                .values(db_signature)
                .execute(&mut conn)
                .unwrap();
            if token_balances.len() > 0 {
                diesel::insert_into(crate::schema::transaction_token_balances::table)
                    .values(token_balances)
                    .execute(&mut conn)
                    .unwrap();
            };
        });

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
            TransactionMeta,
            Option<TransactionError>,
            chrono::NaiveDateTime,
        )>,
        String,
    > {
        let tx = self.cache.get_transaction(id, &signature.to_string())?;
        match tx {
            Some(tx) => {
                // let db
                // let (db_tx, account_keys, instructions, logs, metas, signatures, token_balances) =
                //     transaction_map.into_iter().next().unwrap().1;

                let instructions = tx
                    .instructions
                    .iter()
                    .map(|i| i.to_instruction(tx.account_keys.clone()))
                    .collect::<Vec<Instruction>>();

                let transaction = Transaction {
                    signatures: tx
                        .signatures
                        .into_iter()
                        .map(|s| Signature::from_str(&s.signature).unwrap())
                        .collect(),
                    message: solana_sdk::message::Message::new(&instructions, None),
                };

                let metadata = tx.meta.to_metadata(tx.log_messages, tx.token_balances);

                Ok(Some((
                    transaction,
                    tx.transaction.slot.to_u64().unwrap(),
                    metadata,
                    match tx.meta.to_owned().err {
                        Some(e) => {
                            let deserialized_error: Result<TransactionError, _> =
                                serde_json::from_str(&e);
                            match deserialized_error {
                                Ok(e) => Some(e),
                                Err(_) => Some(TransactionError::InvalidAccountIndex),
                            }
                        }
                        None => None,
                    },
                    tx.transaction.created_at,
                )))
            }
            None => Ok(None),
        }
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
    fn get_transactions_for_address_created_at(
        &self,
        id: Uuid,
        address: &Pubkey,
        start: chrono::NaiveDateTime,
        end: chrono::NaiveDateTime,
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
            .filter(
                crate::schema::transactions::created_at
                    .ge(start)
                    .and(crate::schema::transactions::created_at.le(end)),
            )
            .order(crate::schema::transactions::created_at.asc())
            .select(crate::schema::transactions::all_columns)
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
