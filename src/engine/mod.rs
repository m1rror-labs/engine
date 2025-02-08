use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use chrono::{DateTime, Utc};
use solana_compute_budget::compute_budget::ComputeBudget;
use solana_log_collector::LogCollector;
use solana_program_runtime::{
    invoke_context::{EnvironmentConfig, InvokeContext},
    loaded_programs::ProgramCacheForTxBatch,
    sysvar_cache::SysvarCache,
};
use solana_sdk::{
    account::{Account, AccountSharedData, ReadableAccount, WritableAccount},
    account_utils::StateMut,
    feature_set::{remove_rounding_in_fee_calculation, FeatureSet},
    fee::FeeStructure,
    hash::Hash,
    inner_instruction::{InnerInstruction, InnerInstructionsList},
    instruction::{CompiledInstruction, TRANSACTION_LEVEL_STACK_HEIGHT},
    message::{AddressLoader, SanitizedMessage},
    native_loader, nonce,
    pubkey::Pubkey,
    rent::Rent,
    reserved_account_keys::ReservedAccountKeys,
    signature::Signature,
    system_program,
    sysvar::{self, instructions::construct_instructions_data},
    transaction::{MessageHash, SanitizedTransaction, TransactionError, VersionedTransaction},
    transaction_context::{ExecutionRecord, IndexOfAccount, TransactionContext},
};
use solana_svm::message_processor::MessageProcessor;
use solana_timings::ExecuteTimings;
use transactions::TransactionMetadata;
use uuid::Uuid;

use crate::storage::Storage;

pub mod blocks;
pub mod transactions;

pub trait SVM<T: Storage + AddressLoader> {
    fn new(storage: T) -> Self;
    fn get_account(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<Account>, String>;
    fn get_balance(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<u64>, String>;
    fn latest_blockhash(&self, id: Uuid) -> Result<String, String>;
    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64;
    fn is_blockhash_valid(&self, id: Uuid, blockhash: &Hash) -> Result<bool, String>;
    fn send_transaction(&self, id: Uuid, tx: VersionedTransaction) -> Result<String, String>;
}

pub struct SvmEngine<T: Storage + AddressLoader> {
    rent: Rent,
    fee_structure: FeeStructure,
    feature_set: FeatureSet,
    sysvar_cache: SysvarCache,
    storage: T,
}

impl<T: Storage + AddressLoader> SVM<T> for SvmEngine<T> {
    fn new(storage: T) -> Self {
        SvmEngine {
            rent: Rent::default(),
            fee_structure: FeeStructure::default(),
            feature_set: FeatureSet::all_enabled(),
            sysvar_cache: SysvarCache::default(),
            storage,
        }
    }

    fn get_account(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<Account>, String> {
        self.storage.get_account(id, pubkey)
    }

    fn get_balance(&self, id: Uuid, pubkey: &Pubkey) -> Result<Option<u64>, String> {
        match self.get_account(id, pubkey)? {
            Some(account) => Ok(Some(account.lamports)),
            None => Ok(None),
        }
    }

    fn latest_blockhash(&self, id: Uuid) -> Result<String, String> {
        let block = self.storage.get_latest_block(id)?;

        Ok(block.blockhash.to_string())
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        self.rent.minimum_balance(data_len)
    }

    fn is_blockhash_valid(&self, id: Uuid, blockhash: &Hash) -> Result<bool, String> {
        let block = self.storage.get_block(id, blockhash)?;
        let block_time = match DateTime::from_timestamp(block.block_time as i64, 0) {
            Some(t) => t,
            None => return Err("Invalid block time".to_string()),
        };
        let now = Utc::now();
        let duration = now - block_time;

        Ok(60 <= duration.num_seconds())
    }

    fn send_transaction(&self, id: Uuid, raw_tx: VersionedTransaction) -> Result<String, String> {
        let tx = match SanitizedTransaction::try_create(
            raw_tx,
            MessageHash::Compute,
            Some(false),
            self.storage.clone(),
            &ReservedAccountKeys::empty_key_set(),
        ) {
            Ok(tx) => tx,
            Err(e) => return Err(e.to_string()),
        };

        if !self.is_blockhash_valid(id, tx.message().recent_blockhash())? {
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
            execute_tx_helper(tx, context);
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
        };
        self.storage.save_transaction(id, &meta)?;

        self.storage.set_accounts(
            id,
            post_accounts
                .into_iter()
                .map(|(pubkey, account_shared_data)| (pubkey, Account::from(account_shared_data)))
                .collect(),
        )?;

        Ok("".to_string())
    }
}

impl<T: Storage + AddressLoader> SvmEngine<T> {
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
        match self.accounts.get(pubkey).unwrap() {
            Some(account) => Some(AccountSharedData::from(account.to_owned())),
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
