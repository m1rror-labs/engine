use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_account_decoder::parse_token::{is_known_spl_token_id, UiTokenAmount};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    pubkey::Pubkey,
    transaction::SanitizedTransaction,
};
use spl_token_2022::{
    extension::StateWithExtensions,
    state::{Account as TokenAccount, Mint},
};
use uuid::Uuid;

use crate::storage::Storage;

use super::{
    transactions::{TransactionTokenBalance, TransactionTokenBalancesSet},
    AccountsDB,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: u64,
    pub decimals: u8,
    pub ui_amount: f64,
    pub ui_amount_string: String,
}

pub fn collect_token_balances<T: Storage + Clone + 'static>(
    id: Uuid,
    tx: SanitizedTransaction,
    accounts_db: &AccountsDB,
    storage: T,
    post_accounts: Vec<(Pubkey, AccountSharedData)>,
) -> Option<TransactionTokenBalancesSet> {
    let account_keys = tx.message().account_keys();
    let has_token_program = account_keys.iter().any(is_known_spl_token_id);
    if !has_token_program {
        return None;
    }

    let mut mint_decimals: HashMap<Pubkey, u8> = HashMap::new();

    let mut pre_balances: Vec<TransactionTokenBalance> = Vec::new();
    let mut post_balances: Vec<TransactionTokenBalance> = Vec::new();
    for (index, account_id) in account_keys.iter().enumerate() {
        if tx.message().is_invoked(index) || is_known_spl_token_id(account_id) {
            continue;
        }

        let pre_account = accounts_db.get_account(account_id);
        let post_account = post_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == account_id)
            .map(|(_, account)| account.clone());

        match pre_account {
            Some(pre_account) => {
                if let Some(pre_balance) = collect_token_balance_from_account(
                    id,
                    pre_account,
                    storage.clone(),
                    post_accounts.clone(),
                    index,
                    &mut mint_decimals,
                ) {
                    pre_balances.push(pre_balance);
                }
            }
            None => {}
        };
        match post_account {
            Some(post_account) => {
                if let Some(post_balance) = collect_token_balance_from_account(
                    id,
                    post_account,
                    storage.clone(),
                    post_accounts.clone(),
                    index,
                    &mut mint_decimals,
                ) {
                    post_balances.push(post_balance);
                }
            }
            None => {}
        };
    }

    Some(TransactionTokenBalancesSet {
        pre_token_balances: pre_balances,
        post_token_balances: post_balances,
    })
}

fn collect_token_balance_from_account<T: Storage + Clone + 'static>(
    id: Uuid,
    account: AccountSharedData,
    storage: T,
    post_accounts: Vec<(Pubkey, AccountSharedData)>,
    account_idx: usize,
    mint_decimals: &mut HashMap<Pubkey, u8>,
) -> Option<TransactionTokenBalance> {
    if !is_known_spl_token_id(account.owner()) {
        return None;
    }

    let token_account = StateWithExtensions::<TokenAccount>::unpack(account.data()).ok()?;
    let mint = token_account.base.mint;

    let decimals = mint_decimals.get(&mint).cloned().or_else(|| {
        let decimals = get_mint_decimals(storage, post_accounts, id, &mint)?;
        mint_decimals.insert(mint, decimals);
        Some(decimals)
    })?;

    let ui_amount = token_account.base.amount as f64 / 10f64.powi(decimals as i32);
    Some(TransactionTokenBalance {
        account_index: account_idx as u8,
        mint: mint.to_string(),
        ui_token_amount: UiTokenAmount {
            amount: token_account.base.amount.to_string(),
            decimals,
            ui_amount: Some(ui_amount),
            ui_amount_string: ui_amount.to_string(),
        },
        owner: account.owner().to_string(),
        program_id: token_account.base.owner.to_string(),
    })
}

fn get_mint_decimals<T: Storage + Clone + 'static>(
    storage: T,
    post_accounts: Vec<(Pubkey, AccountSharedData)>,
    id: Uuid,
    mint: &Pubkey,
) -> Option<u8> {
    if mint == &spl_token::native_mint::id() {
        Some(spl_token::native_mint::DECIMALS)
    } else {
        let mint_account = match post_accounts.iter().find(|(pubkey, _)| pubkey == mint) {
            Some((_, account)) => account.clone(),
            None => match storage.get_account(id, mint).ok()? {
                Some(account) => account.to_account_shared_data(),
                None => return None,
            },
        };

        if !is_known_spl_token_id(mint_account.owner()) {
            return None;
        }

        let decimals = StateWithExtensions::<Mint>::unpack(mint_account.data())
            .map(|mint| mint.base.decimals)
            .ok()?;

        Some(decimals)
    }
}
