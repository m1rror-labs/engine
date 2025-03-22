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

pub fn collect_token_balances(
    tx: SanitizedTransaction,
    accounts_db: &AccountsDB,
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

        let pre_account = accounts_db.get_account(account_id).unwrap_or_default();
        let post_account = post_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == account_id)
            .map(|(_, account)| account.clone())
            .unwrap_or_default();

        if let Some(pre_balance) =
            collect_token_balance_from_account(pre_account, accounts_db, index, &mut mint_decimals)
        {
            pre_balances.push(pre_balance);
        }
        if let Some(post_balance) =
            collect_token_balance_from_account(post_account, accounts_db, index, &mut mint_decimals)
        {
            post_balances.push(post_balance);
        }
    }

    None
}

fn collect_token_balance_from_account(
    account: AccountSharedData,
    accounts_db: &AccountsDB,
    account_idx: usize,
    mint_decimals: &mut HashMap<Pubkey, u8>,
) -> Option<TransactionTokenBalance> {
    if !is_known_spl_token_id(account.owner()) {
        return None;
    }

    let token_account = StateWithExtensions::<TokenAccount>::unpack(account.data()).ok()?;
    let mint = token_account.base.mint;

    let decimals = mint_decimals.get(&mint).cloned().or_else(|| {
        let decimals = get_mint_decimals(accounts_db, &mint)?;
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

fn get_mint_decimals(accounts_db: &AccountsDB, mint: &Pubkey) -> Option<u8> {
    if mint == &spl_token::native_mint::id() {
        Some(spl_token::native_mint::DECIMALS)
    } else {
        let mint_account = accounts_db.get_account(mint)?;

        if !is_known_spl_token_id(mint_account.owner()) {
            return None;
        }

        let decimals = StateWithExtensions::<Mint>::unpack(mint_account.data())
            .map(|mint| mint.base.decimals)
            .ok()?;

        Some(decimals)
    }
}
