use actix_web::rt;
use chrono::{DateTime, Utc};
use solana_bpf_loader_program::syscalls::{
    create_program_runtime_environment_v1, create_program_runtime_environment_v2,
};
use solana_compute_budget::compute_budget::ComputeBudget;
use solana_log_collector::LogCollector;
use solana_program::last_restart_slot::LastRestartSlot;
use solana_program_runtime::{
    invoke_context::{EnvironmentConfig, InvokeContext},
    loaded_programs::{LoadProgramMetrics, ProgramCacheEntry, ProgramCacheForTxBatch},
    sysvar_cache::SysvarCache,
};
use solana_sdk::{
    account::{Account, AccountSharedData, ReadableAccount, WritableAccount},
    clock::Clock,
    epoch_rewards::EpochRewards,
    epoch_schedule::EpochSchedule,
    feature_set::{remove_rounding_in_fee_calculation, FeatureSet},
    fee::FeeStructure,
    hash::Hash,
    native_loader,
    pubkey::Pubkey,
    rent::Rent,
    reserved_account_keys::ReservedAccountKeys,
    stake_history::StakeHistory,
    sysvar::{Sysvar, SysvarId},
    transaction::{MessageHash, SanitizedTransaction, TransactionError, VersionedTransaction},
    transaction_context::{IndexOfAccount, TransactionContext},
};
use solana_svm::message_processor::MessageProcessor;
use solana_timings::ExecuteTimings;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{self};
use uuid::Uuid;

use crate::storage::Storage;

use super::{
    blocks::Block, builtins::BUILTINS, construct_instructions_account, execute_tx_helper,
    transactions::TransactionMetadata, validate_fee_payer, AccountsDB, Loader, RentState,
};

#[derive(Clone)]
pub struct TransactionProcessor<T: Storage + Clone + 'static> {
    rent: Rent,
    fee_structure: FeeStructure,
    feature_set: FeatureSet,
    sysvar_cache: SysvarCache,
    storage: T,
    queue_senders: Arc<Mutex<HashMap<Uuid, mpsc::Sender<(Uuid, VersionedTransaction)>>>>,
}

impl<T: Storage + Clone + 'static> TransactionProcessor<T> {
    pub fn new(
        rent: Rent,
        fee_structure: FeeStructure,
        feature_set: FeatureSet,
        sysvar_cache: SysvarCache,
        storage: T,
    ) -> Arc<Self> {
        let mut raw_engine = Self {
            queue_senders: Arc::new(Mutex::new(HashMap::new())),
            rent,
            fee_structure,
            feature_set,
            sysvar_cache,
            storage,
        };
        raw_engine.set_sysvars();
        let engine = Arc::new(raw_engine);

        engine
    }

    pub async fn queue_transaction(&self, id: Uuid, raw_tx: VersionedTransaction) {
        let mut queue_senders = self.queue_senders.lock().unwrap();
        match queue_senders.get(&id) {
            Some(sender) => {
                println!("Queueing transaction");
                if let Err(e) = sender.send((id, raw_tx)).await {
                    println!("Failed to queue transaction: {}", e);
                }
            }
            None => {
                println!("Creating new transaction processor");
                let (sender, mut receiver) = mpsc::channel(100);
                queue_senders.insert(id, sender.clone());

                if let Err(e) = sender.send((id, raw_tx)).await {
                    println!("Failed to queue transaction: {}", e);
                }

                let engine = self.clone();
                rt::spawn(async move {
                    println!("Starting transaction processor");
                    while let Some((id, raw_tx)) = receiver.recv().await {
                        if let Err(e) = engine.process_and_save_transaction(id, raw_tx) {
                            println!("Failed to process transaction: {}", e);
                        }
                    }
                });
            }
        }
    }

    fn set_sysvar<S>(&mut self, sysvar: &S)
    where
        S: Sysvar + SysvarId,
    {
        let account = AccountSharedData::new_data(1, &sysvar, &solana_sdk::sysvar::id()).unwrap();
        self.sysvar_cache.fill_missing_entries(|_, set_sysvar| {
            set_sysvar(account.data());
        });
        self.sysvar_cache.set_sysvar_for_tests(sysvar);
    }

    pub fn set_sysvars(&mut self) {
        self.set_sysvar(&Clock::default());
        self.set_sysvar(&EpochRewards::default());
        self.set_sysvar(&EpochSchedule::default());
        self.set_sysvar(&LastRestartSlot::default());
        self.set_sysvar(&Rent::default());
        // self.set_sysvar(&SlotHistory::default());
        self.set_sysvar(&StakeHistory::default());
    }

    pub fn new_loader(&self, id: Uuid) -> Loader<T> {
        Loader::new(self.storage.clone(), id, self.sysvar_cache.clone())
    }

    fn process_and_save_transaction(
        &self,
        id: Uuid,
        raw_tx: VersionedTransaction,
    ) -> Result<(), String> {
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
        let (current_block, valid_blockhash) =
            self.is_blockhash_valid(id, tx.message().recent_blockhash())?;
        if !valid_blockhash {
            return Err("Blockhash is not valid".to_string());
        };
        let message = tx.message();
        let account_keys = message.account_keys();
        let addresses: Vec<&Pubkey> = account_keys.iter().collect();
        //TODO: I think this works, but maybe not
        let accounts_vec = self.storage.get_accounts(id, &addresses)?;
        println!(
            "Processing transaction with {:?} {:?} accounts",
            addresses.clone(),
            accounts_vec.clone()
        );
        let accounts_map: HashMap<&Pubkey, Option<Account>> = addresses
            .iter()
            .cloned()
            .zip(accounts_vec.into_iter())
            .collect();
        let accounts_db = AccountsDB::new(accounts_map.clone());
        let log_collector = LogCollector::new_ref();
        let (tx_result, accumulated_consume_units, context, fee, payer_key) =
            self.process_transaction(id, &tx, log_collector.clone(), &accounts_db);
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
            execute_tx_helper(tx.clone(), context);
        let Ok(logs) = Rc::try_unwrap(log_collector).map(|lc| lc.into_inner().messages) else {
            unreachable!("Log collector should not be used after send_transaction returns")
        };
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
            pre_accounts: post_accounts
                .clone()
                .iter()
                .map(|(k, _)| {
                    let val = accounts_db.get_account(k).unwrap();

                    (
                        k.to_owned().to_owned(),
                        AccountSharedData::from(val.to_owned()),
                    )
                })
                .collect(),
            post_accounts: post_accounts.clone(),
        };
        self.storage.save_transaction(id, &meta)?;

        self.storage.set_accounts(
            id,
            post_accounts
                .into_iter()
                .map(|(pubkey, account_shared_data)| (pubkey, Account::from(account_shared_data)))
                .collect(),
        )?;

        Ok(())
    }

    pub fn simulate_transaction(
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
        let (current_block, valid_blockhash) =
            self.is_blockhash_valid(id, tx.message().recent_blockhash())?;
        if !valid_blockhash {
            return Err("Blockhash is not valid".to_string());
        };
        let message = tx.message();
        let account_keys = message.account_keys();
        let addresses: Vec<&Pubkey> = account_keys.iter().collect();
        //TODO: I think this works, but maybe not
        let accounts_vec = self.storage.get_accounts(id, &addresses)?;
        println!(
            "simulating transaction with {:?} {:?} accounts",
            addresses.clone(),
            accounts_vec.clone()
        );
        let accounts_map: HashMap<&Pubkey, Option<Account>> = addresses
            .iter()
            .cloned()
            .zip(accounts_vec.into_iter())
            .collect();
        let accounts_db = AccountsDB::new(accounts_map.clone());
        let log_collector = LogCollector::new_ref();
        let (tx_result, accumulated_consume_units, context, _, _) =
            self.process_transaction(id, &tx, log_collector.clone(), &accounts_db);
        if context == None {
            if let Err(err) = tx_result {
                return Err(err.to_string());
            } else {
                return Err("Context is None".to_string());
            }
        }
        if tx_result.is_err() {
            return Err(tx_result.unwrap_err().to_string());
        }
        let context = context.unwrap();
        let (signature, return_data, inner_instructions, post_accounts) =
            execute_tx_helper(tx.clone(), context);
        let Ok(logs) = Rc::try_unwrap(log_collector).map(|lc| lc.into_inner().messages) else {
            unreachable!("Log collector should not be used after send_transaction returns")
        };

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

    fn process_transaction(
        &self,
        id: Uuid,
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
        let mut program_cache_for_tx_batch = ProgramCacheForTxBatch::default();
        BUILTINS.iter().for_each(|builtint| {
            let loaded_program =
                ProgramCacheEntry::new_builtin(0, builtint.name.len(), builtint.entrypoint);
            program_cache_for_tx_batch.replenish(builtint.program_id, Arc::new(loaded_program));
        });
        let program_runtime_v1 = create_program_runtime_environment_v1(
            &self.feature_set,
            &ComputeBudget::default(),
            false,
            true,
        )
        .unwrap();
        let mut mut_self = self.clone();
        mut_self.set_sysvars();

        let program_runtime_v2 =
            create_program_runtime_environment_v2(&ComputeBudget::default(), true);
        program_cache_for_tx_batch.environments.program_runtime_v1 = Arc::new(program_runtime_v1);
        program_cache_for_tx_batch.environments.program_runtime_v2 = Arc::new(program_runtime_v2);
        tx.message().instructions().iter().for_each(|i| {
            let program_id = tx.message().account_keys()[i.program_id_index as usize];
            if BUILTINS.iter().any(|b| b.program_id == program_id) {
                return;
            }
            let program_account = accounts_db.get_account(&program_id).unwrap();
            let program_runtime_v1 = create_program_runtime_environment_v1(
                &self.feature_set,
                &ComputeBudget::default(),
                false,
                true,
            )
            .unwrap();
            let entry = ProgramCacheEntry::new(
                program_account.owner(),
                Arc::new(program_runtime_v1),
                100,
                100,
                program_account.data(),
                program_account.data().len(),
                &mut LoadProgramMetrics::default(),
            )
            .unwrap(); //TODO: This may panic

            program_cache_for_tx_batch.replenish(program_id, Arc::new(entry));
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
                    let owner_account = match accounts_db.get_account(owner_id) {
                        Some(account) => account,
                        None => match self.storage.get_account(id, owner_id) {
                            Ok(account) => match account {
                                Some(account) => account.into(),
                                None => return Err(TransactionError::ProgramAccountNotFound),
                            },
                            Err(_) => {
                                println!("Owner account not found for program {}", owner_id);
                                return Err(TransactionError::ProgramAccountNotFound);
                            }
                        },
                    };
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
                    .map_err(|err| TransactionError::InstructionError(index as u8, err))?
                    .borrow();
                let pubkey = context
                    .get_key_of_account_at_index(index as IndexOfAccount)
                    .map_err(|err| TransactionError::InstructionError(index as u8, err))?;
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

    pub fn is_blockhash_valid(&self, id: Uuid, blockhash: &Hash) -> Result<(Block, bool), String> {
        let block = self.storage.get_block(id, blockhash)?;
        let block_time = match DateTime::from_timestamp(block.block_time as i64, 0) {
            Some(t) => t,
            None => return Err("Invalid block time".to_string()),
        };
        let now = Utc::now();
        let duration = now - block_time;

        Ok((block, 120 >= duration.num_seconds()))
    }
}
