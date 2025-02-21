use actix_web::rt::{self, time};
use blocks::{Block, Blockchain};
use builtins::BUILTINS;
use chrono::{DateTime, Utc};
use engine::TransactionProcessor;
use sha2::{Digest, Sha256};
use solana_banks_interface::{TransactionConfirmationStatus, TransactionStatus};
use solana_compute_budget::compute_budget::ComputeBudget;
use solana_log_collector::LogCollector;
use solana_program::{last_restart_slot::LastRestartSlot, pubkey};
use solana_program_runtime::{
    invoke_context::{EnvironmentConfig, InvokeContext},
    loaded_programs::{ProgramCacheEntry, ProgramCacheForTxBatch},
    sysvar_cache::SysvarCache,
};
use solana_sdk::{
    account::{Account, AccountSharedData, ReadableAccount, WritableAccount},
    account_utils::StateMut,
    address_lookup_table::{self, error::AddressLookupError, state::AddressLookupTable},
    bpf_loader,
    clock::Clock,
    epoch_rewards::EpochRewards,
    epoch_schedule::EpochSchedule,
    feature_set::{remove_rounding_in_fee_calculation, FeatureSet},
    fee::FeeStructure,
    hash::Hash,
    inner_instruction::{InnerInstruction, InnerInstructionsList},
    instruction::{CompiledInstruction, TRANSACTION_LEVEL_STACK_HEIGHT},
    message::{
        v0::{LoadedAddresses, MessageAddressTableLookup},
        AddressLoader, Message, SanitizedMessage, VersionedMessage,
    },
    native_loader,
    native_token::LAMPORTS_PER_SOL,
    nonce,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    reserved_account_keys::ReservedAccountKeys,
    signature::{Keypair, Signature},
    signer::Signer,
    slot_history::SlotHistory,
    stake_history::StakeHistory,
    system_instruction, system_program,
    sysvar::{self, instructions::construct_instructions_data, Sysvar, SysvarId},
    transaction::{
        MessageHash, SanitizedTransaction, Transaction, TransactionError, VersionedTransaction,
    },
    transaction_context::{ExecutionRecord, IndexOfAccount, TransactionContext},
};
use solana_svm::message_processor::MessageProcessor;
use solana_timings::ExecuteTimings;
use spl::load_spl_programs;
use spl_token::state::Account as SplAccount;
use spl_token::state::Mint;
use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr, sync::Arc, time::Duration}; // Add this import at the top of your file
use tokens::TokenAmount;
use transactions::TransactionMetadata;
use uuid::Uuid;

use crate::storage::{transactions::DbTransaction, Storage};

pub mod blocks;
pub mod builtins;
pub mod engine;
pub mod spl;
pub mod tokens;
pub mod transactions;

pub trait SVM<T: Storage + Clone + 'static> {
    fn new(storage: T) -> Self;

    fn create_blockchain(&self, airdrop_keypair: Option<Keypair>) -> Result<Uuid, String>;
    fn get_blockchains(&self) -> Result<Vec<Blockchain>, String>;
    fn delete_blockchain(&self, id: Uuid) -> Result<(), String>;

    fn get_account(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<Account>, String>;
    fn get_transactions_for_address(
        &self,
        id: Uuid,
        pubkey: &Pubkey,
        limit: Option<usize>,
    ) -> Result<Vec<DbTransaction>, String>;
    fn get_balance(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<u64>, String>;
    fn get_block(&self, id: Uuid, slot_number: &u64) -> Result<Option<Block>, String>;
    fn get_latest_block(&self, id: Uuid) -> Result<Block, String>;
    fn get_fee_for_message(&self, message: &SanitizedMessage) -> u64;
    fn get_genesis_hash(&self, id: Uuid) -> Result<Hash, String>;
    fn get_identity(&self, id: Uuid) -> Result<Pubkey, String>;
    fn get_multiple_accounts(
        &self,
        id: Uuid,
        pubkeys: &Vec<&Pubkey>,
    ) -> Result<Vec<Option<Account>>, String>;
    fn latest_blockhash(&self, id: Uuid) -> Result<Block, String>;
    fn current_block(&self, id: Uuid) -> Result<Block, String>;
    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64;
    fn is_blockhash_valid(&self, id: Uuid, blockhash: &Hash) -> Result<(Block, bool), String>;
    fn get_token_accounts_by_owner(
        &self,
        id: Uuid,
        pubkey: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String>;
    fn get_token_supply(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<TokenAmount>, String>;
    fn get_token_account_balance(
        &self,
        id: Uuid,
        pubkey: &Pubkey,
    ) -> Result<Option<TokenAmount>, String>;
    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<Option<(Transaction, TransactionStatus)>, String>;
    fn get_transaction_count(&self, id: Uuid) -> Result<u64, String>;
    fn send_transaction(&self, id: Uuid, tx: VersionedTransaction) -> Result<String, String>;
    fn simulate_transaction(
        &self,
        id: Uuid,
        tx: VersionedTransaction,
    ) -> Result<TransactionMetadata, String>;
    fn airdrop(&self, id: Uuid, pubkey: &Pubkey, lamports: u64) -> Result<String, String>;
    fn add_program(&self, id: Uuid, program_id: Pubkey, program_bytes: &[u8])
        -> Result<(), String>;

    #[allow(async_fn_in_trait)]
    async fn signature_subscribe(
        &self,
        id: Uuid,
        signature: &Signature,
        commitment: TransactionConfirmationStatus,
    ) -> Result<u64, String>;
}

pub struct SvmEngine<T: Storage + Clone + 'static> {
    rent: Rent,
    fee_structure: FeeStructure,
    feature_set: FeatureSet,
    sysvar_cache: SysvarCache,
    storage: T,
    transaction_processor: Arc<TransactionProcessor<T>>,
}

impl<T: Storage + Clone + 'static> SVM<T> for SvmEngine<T> {
    fn new(storage: T) -> Self {
        let tx_processor = TransactionProcessor::new(
            Rent::default(),
            FeeStructure::default(),
            FeatureSet::all_enabled(),
            SysvarCache::default(),
            storage.clone(),
        );
        let mut engine = SvmEngine {
            rent: Rent::default(),
            fee_structure: FeeStructure::default(),
            feature_set: FeatureSet::all_enabled(),
            sysvar_cache: SysvarCache::default(),
            storage: storage,
            transaction_processor: tx_processor,
        };
        engine.set_sysvars();

        // let cloned_processor = engine.transaction_processor.clone();
        // rt::spawn(async move {
        //     cloned_processor.clone().start_processing();
        // });

        engine
    }

    async fn signature_subscribe(
        &self,
        id: Uuid,
        signature: &Signature,
        commitment: TransactionConfirmationStatus,
    ) -> Result<u64, String> {
        let mut interval = time::interval(Duration::from_millis(400));
        loop {
            interval.tick().await;
            let tx = self.get_transaction(id, signature)?;
            println!("Checking transaction: {:?}, {:?}", signature, tx);
            if tx == None {
                continue;
            }
            if let Some((_, status)) = tx {
                println!("Transaction status: {:?}", status);
                if status.confirmation_status == Some(commitment.clone()) {
                    return Ok(status.slot);
                }
            }
        }
    }

    fn create_blockchain(&self, airdrop_keypair: Option<Keypair>) -> Result<Uuid, String> {
        let keypair = match airdrop_keypair {
            Some(k) => k,
            None => Keypair::new(),
        };

        let blockchain = Blockchain {
            id: Uuid::new_v4(),
            created_at: Utc::now().naive_utc(),
            airdrop_keypair: keypair.insecure_clone(),
        };

        let id = self.storage.set_blockchain(&blockchain)?;

        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        let hash_array = hasher.finalize();
        let hash = Hash::new_from_array(hash_array.into());
        self.storage.set_block(
            id,
            &Block {
                blockhash: hash,
                block_time: 0,
                previous_blockhash: Hash::default(),
                block_height: 0,
                parent_slot: 0,
                transactions: vec![],
            },
        )?;
        self.storage.set_account(
            id,
            &keypair.pubkey(),
            Account {
                lamports: 1_000_000u64.wrapping_mul(LAMPORTS_PER_SOL),
                data: vec![],
                owner: system_program::id(),
                executable: false,
                rent_epoch: 1000000,
            },
            None,
        )?;
        BUILTINS.iter().for_each(|builtint| {
            let mut account: Account =
                native_loader::create_loadable_account_for_test(builtint.name).into();
            account.rent_epoch = 1000000;
            self.storage
                .set_account(id, &builtint.program_id, account, None)
                .expect("Failed to set builtin account");
        });
        load_spl_programs(self, id)?;

        Ok(id)
    }

    fn delete_blockchain(&self, id: Uuid) -> Result<(), String> {
        self.storage.delete_blockchain(id)
    }

    fn get_blockchains(&self) -> Result<Vec<Blockchain>, String> {
        self.storage.get_blockchains()
    }

    fn get_account(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<Account>, String> {
        self.storage.get_account(id, pubkey)
    }

    fn get_transactions_for_address(
        &self,
        id: Uuid,
        pubkey: &Pubkey,
        limit: Option<usize>,
    ) -> Result<Vec<DbTransaction>, String> {
        self.storage.get_transactions_for_address(id, pubkey, limit)
    }

    fn get_balance(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<u64>, String> {
        match self.get_account(id, pubkey)? {
            Some(account) => Ok(Some(account.lamports)),
            None => Ok(None),
        }
    }

    fn get_block(&self, id: Uuid, slot_number: &u64) -> Result<Option<Block>, String> {
        self.storage.get_block_by_height(id, slot_number.to_owned())
    }

    fn get_latest_block(&self, id: Uuid) -> Result<Block, String> {
        self.storage.get_latest_block(id)
    }

    fn get_fee_for_message(&self, message: &SanitizedMessage) -> u64 {
        solana_fee::calculate_fee(
            message,
            false,
            self.fee_structure.lamports_per_signature,
            0,
            self.feature_set
                .is_active(&remove_rounding_in_fee_calculation::id()),
        )
    }

    fn get_genesis_hash(&self, id: Uuid) -> Result<Hash, String> {
        let block = self.get_block(id, &0)?;
        match block {
            Some(block) => Ok(block.blockhash),
            None => Err("Genesis block not found".to_string()),
        }
    }

    fn get_identity(&self, id: Uuid) -> Result<Pubkey, String> {
        let blockchain = self.storage.get_blockchain(id)?;
        Ok(blockchain.airdrop_keypair.pubkey())
    }

    fn get_multiple_accounts(
        &self,
        id: Uuid,
        pubkeys: &Vec<&Pubkey>,
    ) -> Result<Vec<Option<Account>>, String> {
        self.storage.get_accounts(id, pubkeys)
    }

    fn latest_blockhash(&self, id: Uuid) -> Result<Block, String> {
        let block = self.storage.get_latest_block(id)?;

        // if self.is_blockhash_valid(id, &block.blockhash)? {
        //     return Ok(block);
        // }

        let mut hasher = Sha256::new();
        hasher.update(block.blockhash.as_ref());
        let hash_array = hasher.finalize();
        let current_blockhash = Hash::new_from_array(hash_array.into());
        let next_block = Block {
            blockhash: current_blockhash,
            block_time: block.block_time + 60,
            previous_blockhash: block.blockhash,
            block_height: block.block_height + 1,
            parent_slot: block.block_height,
            transactions: vec![],
        };

        self.storage.set_block(id, &next_block)?;

        Ok(next_block)
    }

    fn current_block(&self, id: Uuid) -> Result<Block, String> {
        let block = self.storage.get_latest_block(id)?;
        Ok(block)
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        self.rent.minimum_balance(data_len)
    }

    fn is_blockhash_valid(&self, id: Uuid, blockhash: &Hash) -> Result<(Block, bool), String> {
        let block = self.storage.get_block(id, blockhash)?;
        let block_time = match DateTime::from_timestamp(block.block_time as i64, 0) {
            Some(t) => t,
            None => return Err("Invalid block time".to_string()),
        };
        let now = Utc::now();
        let duration = now - block_time;

        Ok((block, 120 >= duration.num_seconds()))
    }

    fn get_token_account_balance(
        &self,
        id: Uuid,
        pubkey: &Pubkey,
    ) -> Result<Option<TokenAmount>, String> {
        let account = self.get_account(id, pubkey)?;
        if let None = account {
            return Ok(None);
        }
        let account = account.unwrap();
        let spl =
            SplAccount::unpack_from_slice(account.data.as_slice()).map_err(|e| e.to_string())?;
        let mint = self.get_account(id, &spl.mint)?;
        if let None = mint {
            return Ok(None);
        }
        let mint = mint.unwrap();
        let mint = Mint::unpack_from_slice(mint.data.as_slice()).map_err(|e| e.to_string())?;
        Ok(Some(TokenAmount {
            amount: spl.amount,
            decimals: mint.decimals,
            ui_amount: spl.amount as f64 / 10f64.powf(mint.decimals as f64),
            ui_amount_string: (spl.amount as f64 / 10f64.powf(mint.decimals as f64)).to_string(),
        }))
    }

    fn get_token_accounts_by_owner(
        &self,
        id: Uuid,
        pubkey: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, String> {
        let token_program = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        let token_2022 = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
        let token_accounts = self
            .storage
            .get_token_accounts_by_owner(id, pubkey, &token_program);
        let token_2022_accounts = self
            .storage
            .get_token_accounts_by_owner(id, pubkey, &token_2022);
        let mut accounts = token_accounts?;
        accounts.extend(token_2022_accounts?);
        Ok(accounts)
    }

    fn get_token_supply(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<TokenAmount>, String> {
        let account = self.get_account(id, pubkey)?;
        if let None = account {
            return Ok(None);
        }
        let account = account.unwrap();

        Mint::unpack_from_slice(account.data.as_slice()).map_or_else(
            |_| Ok(None),
            |mint| {
                Ok(Some(TokenAmount {
                    amount: mint.supply,
                    decimals: mint.decimals,
                    ui_amount: mint.supply as f64 / 10f64.powf(mint.decimals as f64),
                    ui_amount_string: (mint.supply as f64 / 10f64.powf(mint.decimals as f64))
                        .to_string(),
                }))
            },
        )
    }

    fn get_transaction(
        &self,
        id: Uuid,
        signature: &Signature,
    ) -> Result<Option<(Transaction, TransactionStatus)>, String> {
        let res = match self.storage.get_transaction(id, signature) {
            Ok(res) => res,
            Err(_) => return Ok(None),
        };
        if res == None {
            return Ok(None);
        }
        let (tx, slot, tx_res, created_at) = res.unwrap();
        Ok(Some((
            tx,
            TransactionStatus {
                slot,
                confirmations: None,
                err: tx_res,
                confirmation_status: Some(tx_confirmation_status(created_at.and_utc())),
            },
        )))
    }

    fn get_transaction_count(&self, id: Uuid) -> Result<u64, String> {
        self.storage.get_transaction_count(id)
    }

    fn send_transaction(&self, id: Uuid, raw_tx: VersionedTransaction) -> Result<String, String> {
        let tx_processor = self.transaction_processor.clone();
        let tx_clone = raw_tx.clone();
        if raw_tx.signatures.len() < 1 {
            return Err("Transaction must include signatures".to_string());
        }
        if self
            .storage
            .get_transaction(id, &raw_tx.signatures[0])?
            .is_some()
        {
            return Err("Transaction cannot be replayed".to_string());
        };

        rt::spawn(async move {
            tx_processor.queue_transaction(id, tx_clone).await;
        });

        Ok(raw_tx.signatures[0].to_string())
    }

    fn simulate_transaction(
        &self,
        id: Uuid,
        raw_tx: VersionedTransaction,
    ) -> Result<TransactionMetadata, String> {
        let address_loader = Loader::new(self.storage.clone(), id, self.sysvar_cache.clone());

        let tx = match SanitizedTransaction::try_create(
            raw_tx,
            MessageHash::Compute,
            Some(false),
            address_loader,
            &ReservedAccountKeys::empty_key_set(),
        ) {
            Ok(tx) => tx,
            Err(e) => return Err(e.to_string()),
        };
        let (_, valid_blockhash) = self.is_blockhash_valid(id, tx.message().recent_blockhash())?;
        if !valid_blockhash {
            return Err("Blockhash is not valid".to_string());
        };
        if self.storage.get_transaction(id, tx.signature())?.is_some() {
            return Err("Transaction cannot be replayed".to_string());
        };

        let message = tx.message();
        let account_keys = message.account_keys();
        let addresses = account_keys.iter().collect();
        //TODO: I think this works, but maybe not
        let accounts_vec = self.storage.get_accounts(id, &addresses)?;
        let accounts_map: HashMap<&Pubkey, Option<Account>> = addresses
            .iter()
            .cloned()
            .zip(accounts_vec.into_iter())
            .collect();
        let accounts_db = AccountsDB::new(accounts_map.clone());
        let log_collector = LogCollector::new_ref();
        let (tx_result, accumulated_consume_units, context, fee, payer_key) =
            self.process_transaction(&tx, log_collector.clone(), &accounts_db);
        if context == None {
            if let Err(err) = tx_result {
                return Err(err.to_string());
            } else {
                return Err("Context is None".to_string());
            }
        }
        //Decrement account if tx failed and payer is not None
        if tx_result.is_err() && payer_key.is_some() {
            let payer_key = payer_key.unwrap();
            let payer_account = accounts_db.get_account(&payer_key).unwrap();
            payer_account.to_owned().checked_sub_lamports(fee).unwrap();
            self.storage
                .set_account_lamports(id, &payer_key, payer_account.lamports())?;
        }

        let context = context.unwrap();
        let (signature, return_data, inner_instructions, post_accounts) =
            execute_tx_helper(tx.clone(), context.clone());
        let Ok(logs) = Rc::try_unwrap(log_collector).map(|lc| lc.into_inner().messages) else {
            unreachable!("Log collector should not be used after send_transaction returns")
        };

        let current_block = self.current_block(id)?;
        let meta = TransactionMetadata {
            signature,
            err: tx_result.err(),
            logs,
            inner_instructions,
            compute_units_consumed: accumulated_consume_units,
            return_data,
            tx: tx.clone(),
            current_block,
            //TODO: This may be wrong
            pre_accounts: accounts_db
                .accounts
                .iter()
                .map(|(k, v)| {
                    if let Some(account) = v {
                        (
                            k.to_owned().to_owned(),
                            AccountSharedData::from(account.to_owned()),
                        )
                    } else {
                        (k.to_owned().to_owned(), AccountSharedData::default())
                    }
                })
                .collect(),
            post_accounts: post_accounts.clone(),
        };

        Ok(meta)
    }

    fn airdrop(&self, id: Uuid, pubkey: &Pubkey, lamports: u64) -> Result<String, String> {
        let blockchain = self.storage.get_blockchain(id)?;
        let payer = blockchain.airdrop_keypair;
        let latest_blockhash = self.latest_blockhash(id)?;
        let latest_blockhash = latest_blockhash.blockhash.to_string();
        let tx = VersionedTransaction::try_new(
            VersionedMessage::Legacy(Message::new_with_blockhash(
                &[system_instruction::transfer(
                    &payer.pubkey(),
                    pubkey,
                    lamports,
                )],
                Some(&payer.pubkey()),
                &Hash::from_str(latest_blockhash.as_str()).unwrap(),
            )),
            &[payer],
        )
        .unwrap();

        self.send_transaction(id, tx)
    }

    fn add_program(
        &self,
        id: Uuid,
        program_id: Pubkey,
        program_bytes: &[u8],
    ) -> Result<(), String> {
        let program_len = program_bytes.len();
        let lamports = self.minimum_balance_for_rent_exemption(program_len);
        let account = Account {
            lamports,
            data: program_bytes.to_vec(),
            owner: bpf_loader::id(),
            executable: true,
            rent_epoch: 100000000,
        };
        self.storage.set_account(id, &program_id, account, None)?;
        Ok(())
    }
}

impl<T: Storage + Clone + 'static> SvmEngine<T> {
    /// Sets the sysvar to the test environment.
    pub fn set_sysvar<S>(&mut self, sysvar: &S)
    where
        S: Sysvar + SysvarId,
    {
        let account = AccountSharedData::new_data(1, &sysvar, &solana_sdk::sysvar::id()).unwrap();
        self.sysvar_cache.fill_missing_entries(|_, set_sysvar| {
            set_sysvar(account.data());
        });
    }

    fn set_sysvars(&mut self) {
        self.set_sysvar(&Clock::default());
        self.set_sysvar(&EpochRewards::default());
        self.set_sysvar(&EpochSchedule::default());
        self.set_sysvar(&LastRestartSlot::default());
        self.set_sysvar(&Rent::default());
        self.set_sysvar(&SlotHistory::default());
        self.set_sysvar(&StakeHistory::default());
    }

    fn create_transaction_context(
        &self,
        compute_budget: ComputeBudget,
        accounts: Vec<(Pubkey, AccountSharedData)>,
    ) -> TransactionContext {
        TransactionContext::new(
            accounts,
            self.rent.clone(),
            compute_budget.max_instruction_stack_depth,
            compute_budget.max_instruction_trace_length,
        )
    }

    fn check_accounts_rent(
        &self,
        tx: &SanitizedTransaction,
        context: &TransactionContext,
        accounts_db: &AccountsDB,
    ) -> Result<(), TransactionError> {
        for index in 0..tx.message().account_keys().len() {
            if tx.message().is_writable(index) {
                let account = context
                    .get_account_at_index(index as IndexOfAccount)
                    .map_err(|err| {
                        println!("Error getting account at index: {:?}", err);
                        TransactionError::InstructionError(index as u8, err)
                    })?
                    .borrow();
                let pubkey = context
                    .get_key_of_account_at_index(index as IndexOfAccount)
                    .map_err(|err| {
                        println!("Error getting key of account at index: {:?}", err);
                        TransactionError::InstructionError(index as u8, err)
                    })?;
                let rent = self.sysvar_cache.get_rent().unwrap_or_default();

                if !account.data().is_empty() {
                    let post_rent_state = RentState::from_account(&account, &rent);
                    let pre_rent_state = RentState::from_account(
                        &accounts_db.get_account(pubkey).unwrap_or_default(),
                        &rent,
                    );

                    if !post_rent_state.transition_allowed_from(&pre_rent_state) {
                        return Err(TransactionError::InsufficientFundsForRent {
                            account_index: index as u8,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn process_transaction(
        &self,
        tx: &SanitizedTransaction,
        log_collector: Rc<RefCell<LogCollector>>,
        accounts_db: &AccountsDB,
    ) -> (
        Result<(), TransactionError>,
        u64,
        Option<TransactionContext>,
        u64,
        Option<Pubkey>,
    ) {
        let compute_budget = ComputeBudget::default();
        let blockhash = tx.message().recent_blockhash();
        //TODO: I dont think I need to do anything here, but if something goes wrong, look here
        let mut program_cache_for_tx_batch = ProgramCacheForTxBatch::default();
        BUILTINS.iter().for_each(|builtint| {
            let loaded_program =
                ProgramCacheEntry::new_builtin(0, builtint.name.len(), builtint.entrypoint);
            program_cache_for_tx_batch.replenish(builtint.program_id, Arc::new(loaded_program));
        });
        let mut accumulated_consume_units = 0;
        let message = tx.message();
        let account_keys = message.account_keys();
        let fee = solana_fee::calculate_fee(
            message,
            false,
            self.fee_structure.lamports_per_signature,
            0,
            self.feature_set
                .is_active(&remove_rounding_in_fee_calculation::id()),
        );
        let mut validated_fee_payer = false;
        let mut payer_key = None;
        let maybe_accounts = account_keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let mut account_found = true;
                let account = if solana_sdk::sysvar::instructions::check_id(key) {
                    construct_instructions_account(message)
                } else {
                    let mut account = accounts_db.get_account(key).unwrap_or_else(|| {
                        account_found = false;
                        let mut default_account = AccountSharedData::default();
                        default_account.set_rent_epoch(0);
                        default_account
                    });
                    if !validated_fee_payer
                        && (!message.is_invoked(i) || message.is_instruction_account(i))
                    {
                        validate_fee_payer(
                            key,
                            &mut account,
                            i as IndexOfAccount,
                            &self.sysvar_cache.get_rent().unwrap(),
                            fee,
                        )?;
                        validated_fee_payer = true;
                        payer_key = Some(*key);
                    }
                    account
                };

                Ok((*key, account))
            })
            .collect::<solana_sdk::transaction::Result<Vec<_>>>();
        let mut accounts = match maybe_accounts {
            Ok(accs) => accs,
            Err(e) => {
                return (Err(e), accumulated_consume_units, None, fee, payer_key);
            }
        };
        if !validated_fee_payer {
            return (
                Err(TransactionError::AccountNotFound),
                accumulated_consume_units,
                None,
                fee,
                payer_key,
            );
        }
        let builtins_start_index = accounts.len();
        let maybe_program_indices = tx
            .message()
            .instructions()
            .iter()
            .map(|c| {
                let mut account_indices: Vec<u16> = Vec::with_capacity(2);
                let program_index = c.program_id_index as usize;
                // This may never error, because the transaction is sanitized
                let (program_id, program_account) = accounts.get(program_index).unwrap();
                if native_loader::check_id(program_id) {
                    return Ok(account_indices);
                }
                if !program_account.executable() {
                    return Err(TransactionError::InvalidProgramForExecution);
                }
                account_indices.insert(0, program_index as IndexOfAccount);

                let owner_id = program_account.owner();
                if native_loader::check_id(owner_id) {
                    return Ok(account_indices);
                }
                if !accounts
                    .get(builtins_start_index..)
                    .ok_or(TransactionError::ProgramAccountNotFound)?
                    .iter()
                    .any(|(key, _)| key == owner_id)
                {
                    let owner_account = accounts_db.get_account(owner_id).unwrap();
                    if !native_loader::check_id(owner_account.owner()) {
                        return Err(TransactionError::InvalidProgramForExecution);
                    }
                    if !owner_account.executable() {
                        return Err(TransactionError::InvalidProgramForExecution);
                    }
                    accounts.push((*owner_id, owner_account.into()));
                }
                Ok(account_indices)
            })
            .collect::<Result<Vec<Vec<u16>>, TransactionError>>();
        match maybe_program_indices {
            Ok(program_indices) => {
                let mut context = self.create_transaction_context(compute_budget, accounts);
                let mut tx_result = MessageProcessor::process_message(
                    tx.message(),
                    &program_indices,
                    &mut InvokeContext::new(
                        &mut context,
                        &mut program_cache_for_tx_batch,
                        EnvironmentConfig::new(
                            *blockhash,
                            None,
                            None,
                            Arc::new(self.feature_set.clone().into()),
                            0,
                            &self.sysvar_cache,
                        ),
                        Some(log_collector),
                        compute_budget,
                    ),
                    &mut ExecuteTimings::default(),
                    &mut accumulated_consume_units,
                )
                .map(|_| ());
                println!("Transaction result: {:?}", tx_result);
                if let Err(err) = self.check_accounts_rent(tx, &context, accounts_db) {
                    tx_result = Err(err);
                };

                (
                    tx_result,
                    accumulated_consume_units,
                    Some(context),
                    fee,
                    payer_key,
                )
            }
            Err(e) => (Err(e), accumulated_consume_units, None, fee, payer_key),
        }
    }

    pub fn progress_block(&self, id: Uuid) -> Result<(), String> {
        let latest_block = self.storage.get_latest_block(id)?;

        let mut hasher = Sha256::new();
        hasher.update(latest_block.blockhash.as_ref());
        let hash_array = hasher.finalize();
        let current_blockhash = Hash::new_from_array(hash_array.into());
        let next_block = Block {
            blockhash: current_blockhash,
            block_time: latest_block.block_time + 60,
            previous_blockhash: latest_block.blockhash,
            block_height: latest_block.block_height + 1,
            parent_slot: latest_block.block_height,
            transactions: vec![],
        };

        self.storage.set_block(id, &next_block)?;
        Ok(())
    }
}

pub fn construct_instructions_account(message: &SanitizedMessage) -> AccountSharedData {
    AccountSharedData::from(Account {
        data: construct_instructions_data(&message.decompile_instructions()),
        owner: sysvar::id(),
        ..Account::default()
    })
}

//this code is taken from https://github.com/solana-labs/solana/blob/master/runtime/src/accounts/account_rent_state.rs

#[derive(Debug, PartialEq, Eq)]
pub enum RentState {
    /// account.lamports == 0
    Uninitialized,
    /// 0 < account.lamports < rent-exempt-minimum
    RentPaying {
        lamports: u64,    // account.lamports()
        data_size: usize, // account.data().len()
    },
    /// account.lamports >= rent-exempt-minimum
    RentExempt,
}

impl RentState {
    pub fn from_account(account: &AccountSharedData, rent: &Rent) -> Self {
        if account.lamports() == 0 {
            Self::Uninitialized
        } else if rent.is_exempt(account.lamports(), account.data().len()) {
            Self::RentExempt
        } else {
            Self::RentPaying {
                data_size: account.data().len(),
                lamports: account.lamports(),
            }
        }
    }

    pub fn transition_allowed_from(&self, pre_rent_state: &RentState) -> bool {
        match self {
            Self::Uninitialized | Self::RentExempt => true,
            Self::RentPaying {
                data_size: post_data_size,
                lamports: post_lamports,
            } => {
                match pre_rent_state {
                    Self::Uninitialized | Self::RentExempt => false,
                    Self::RentPaying {
                        data_size: pre_data_size,
                        lamports: pre_lamports,
                    } => {
                        // Cannot remain RentPaying if resized or credited.
                        post_data_size == pre_data_size && post_lamports <= pre_lamports
                    }
                }
            }
        }
    }
}

// modified version of the private fn in solana-svm
fn check_rent_state_with_account(
    pre_rent_state: &RentState,
    post_rent_state: &RentState,
    address: &Pubkey,
    account_index: IndexOfAccount,
) -> solana_sdk::transaction::Result<()> {
    if !solana_sdk::incinerator::check_id(address)
        && !post_rent_state.transition_allowed_from(pre_rent_state)
    {
        let account_index = account_index as u8;
        Err(TransactionError::InsufficientFundsForRent { account_index })
    } else {
        Ok(())
    }
}

/// Lighter version of the one in the solana-svm crate.
///
/// Check whether the payer_account is capable of paying the fee. The
/// side effect is to subtract the fee amount from the payer_account
/// balance of lamports. If the payer_acount is not able to pay the
/// fee a specific error is returned.
fn validate_fee_payer(
    payer_address: &Pubkey,
    payer_account: &mut AccountSharedData,
    payer_index: IndexOfAccount,
    rent: &Rent,
    fee: u64,
) -> solana_sdk::transaction::Result<()> {
    if payer_account.lamports() == 0 {
        return Err(TransactionError::AccountNotFound);
    }
    let system_account_kind = get_system_account_kind(payer_account)
        .ok_or_else(|| TransactionError::InvalidAccountForFee)?;
    let min_balance = match system_account_kind {
        SystemAccountKind::System => 0,
        SystemAccountKind::Nonce => {
            // Should we ever allow a fees charge to zero a nonce account's
            // balance. The state MUST be set to uninitialized in that case
            rent.minimum_balance(solana_sdk::nonce::State::size())
        }
    };

    let payer_lamports = payer_account.lamports();

    payer_lamports
        .checked_sub(min_balance)
        .and_then(|v| v.checked_sub(fee))
        .ok_or_else(|| TransactionError::InsufficientFundsForFee)?;

    let payer_pre_rent_state = RentState::from_account(payer_account, rent);
    // we already checked above if we have sufficient balance so this should never error.
    payer_account.checked_sub_lamports(fee).unwrap();

    let payer_post_rent_state = RentState::from_account(payer_account, rent);
    check_rent_state_with_account(
        &payer_pre_rent_state,
        &payer_post_rent_state,
        payer_address,
        payer_index,
    )
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SystemAccountKind {
    System,
    Nonce,
}

pub fn get_system_account_kind(account: &AccountSharedData) -> Option<SystemAccountKind> {
    if system_program::check_id(account.owner()) {
        if account.data().is_empty() {
            Some(SystemAccountKind::System)
        } else if account.data().len() == nonce::State::size() {
            let nonce_versions: nonce::state::Versions = account.state().ok()?;
            match nonce_versions.state() {
                nonce::State::Uninitialized => None,
                nonce::State::Initialized(_) => Some(SystemAccountKind::Nonce),
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub struct AccountsDB<'a> {
    accounts: HashMap<&'a Pubkey, Option<Account>>,
}

impl<'a> AccountsDB<'a> {
    fn new(accounts: HashMap<&'a Pubkey, Option<Account>>) -> Self {
        AccountsDB { accounts }
    }

    fn get_account(&self, pubkey: &Pubkey) -> Option<AccountSharedData> {
        match self.accounts.get(pubkey) {
            Some(account) => match account {
                Some(account) => Some(AccountSharedData::from(account.to_owned())),
                None => None,
            },
            None => None,
        }
    }
}

fn execute_tx_helper(
    sanitized_tx: SanitizedTransaction,
    ctx: TransactionContext,
) -> (
    Signature,
    solana_sdk::transaction_context::TransactionReturnData,
    InnerInstructionsList,
    Vec<(Pubkey, AccountSharedData)>,
) {
    let signature = sanitized_tx.signature().to_owned();
    let inner_instructions = inner_instructions_list_from_instruction_trace(&ctx);
    let ExecutionRecord {
        accounts,
        return_data,
        touched_account_count: _,
        accounts_resize_delta: _,
    } = ctx.into();
    let msg = sanitized_tx.message();
    let post_accounts = accounts
        .into_iter()
        .enumerate()
        .filter_map(|(idx, pair)| msg.is_writable(idx).then_some(pair))
        .collect();
    (signature, return_data, inner_instructions, post_accounts)
}

/// Pulled verbatim from `solana-svm` crate, `transaction_processor.rs`
pub fn inner_instructions_list_from_instruction_trace(
    transaction_context: &TransactionContext,
) -> InnerInstructionsList {
    debug_assert!(transaction_context
        .get_instruction_context_at_index_in_trace(0)
        .map(|instruction_context| instruction_context.get_stack_height()
            == TRANSACTION_LEVEL_STACK_HEIGHT)
        .unwrap_or(true));
    let mut outer_instructions = Vec::new();
    for index_in_trace in 0..transaction_context.get_instruction_trace_length() {
        if let Ok(instruction_context) =
            transaction_context.get_instruction_context_at_index_in_trace(index_in_trace)
        {
            let stack_height = instruction_context.get_stack_height();
            if stack_height == TRANSACTION_LEVEL_STACK_HEIGHT {
                outer_instructions.push(Vec::new());
            } else if let Some(inner_instructions) = outer_instructions.last_mut() {
                let stack_height = u8::try_from(stack_height).unwrap_or(u8::MAX);
                let instruction = CompiledInstruction::new_from_raw_parts(
                    instruction_context
                        .get_index_of_program_account_in_transaction(
                            instruction_context
                                .get_number_of_program_accounts()
                                .saturating_sub(1),
                        )
                        .unwrap_or_default() as u8,
                    instruction_context.get_instruction_data().to_vec(),
                    (0..instruction_context.get_number_of_instruction_accounts())
                        .map(|instruction_account_index| {
                            instruction_context
                                .get_index_of_instruction_account_in_transaction(
                                    instruction_account_index,
                                )
                                .unwrap_or_default() as u8
                        })
                        .collect(),
                );
                inner_instructions.push(InnerInstruction {
                    instruction,
                    stack_height,
                });
            } else {
                debug_assert!(false);
            }
        } else {
            debug_assert!(false);
        }
    }
    outer_instructions
}

#[derive(Clone)]
struct Loader<T: Storage + Clone + 'static> {
    storage: T,
    id: Uuid,
    sysvar_cache: SysvarCache,
}

impl<T: Storage + Clone + 'static> AddressLoader for Loader<T> {
    fn load_addresses(
        self,
        lookups: &[solana_sdk::message::v0::MessageAddressTableLookup],
    ) -> Result<solana_sdk::message::v0::LoadedAddresses, solana_sdk::message::AddressLoaderError>
    {
        lookups
            .iter()
            .map(|lookup| {
                self.load_lookup_table_addresses(lookup).map_err(|_| {
                    solana_sdk::message::AddressLoaderError::LookupTableAccountNotFound
                })
            })
            .collect()
    }
}

impl<T: Storage + Clone + 'static> Loader<T> {
    fn new(storage: T, id: Uuid, sysvar_cache: SysvarCache) -> Self {
        Loader {
            storage,
            id,
            sysvar_cache,
        }
    }

    fn load_lookup_table_addresses(
        &self,
        address_table_lookup: &MessageAddressTableLookup,
    ) -> std::result::Result<LoadedAddresses, AddressLookupError> {
        let table_account = self
            .storage
            .get_account(self.id, &address_table_lookup.account_key)
            .map_err(|_| AddressLookupError::LookupTableAccountNotFound)?
            .ok_or(AddressLookupError::LookupTableAccountNotFound)?;

        if table_account.owner() == &address_lookup_table::program::id() {
            let slot_hashes = self.sysvar_cache.get_slot_hashes().unwrap();
            let current_slot = self.sysvar_cache.get_clock().unwrap().slot;
            let lookup_table = AddressLookupTable::deserialize(table_account.data())
                .map_err(|_ix_err| AddressLookupError::InvalidAccountData)?;

            Ok(LoadedAddresses {
                writable: lookup_table.lookup(
                    current_slot,
                    &address_table_lookup.writable_indexes,
                    &slot_hashes,
                )?,
                readonly: lookup_table.lookup(
                    current_slot,
                    &address_table_lookup.readonly_indexes,
                    &slot_hashes,
                )?,
            })
        } else {
            Err(AddressLookupError::InvalidAccountOwner)
        }
    }
}

fn tx_confirmation_status(time: chrono::DateTime<Utc>) -> TransactionConfirmationStatus {
    let now = Utc::now();
    let duration = now - time;
    if duration.num_seconds() > 1 && duration.num_seconds() <= 2 {
        TransactionConfirmationStatus::Confirmed
    } else if duration.num_seconds() > 3 {
        TransactionConfirmationStatus::Finalized
    } else {
        TransactionConfirmationStatus::Processed
    }
}
