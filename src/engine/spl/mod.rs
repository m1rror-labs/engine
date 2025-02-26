use solana_program::pubkey;
use token22::TOKEN_2022_BASE_64_STR;
use uuid::Uuid;
use base64::prelude::*;

use crate::storage::Storage;
mod token22;

use super::{SvmEngine, SVM};

pub fn load_spl_programs<T: Storage + Clone + 'static>(svm: &SvmEngine<T>, id: Uuid) -> Result<(), String> {
    let token_2022_bytes = match BASE64_STANDARD.decode(TOKEN_2022_BASE_64_STR) {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("Failed to decode token22 base64: {}", e));
        }
    };

    svm.add_program(
        id,
        pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        include_bytes!("programs/spl_token-3.5.0.so"),
    )?;
    svm.add_program(
        id,
        pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
        &token_2022_bytes,
    )?;
    svm.add_program(
        id,
        pubkey!("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo"),
        include_bytes!("programs/spl_memo-1.0.0.so"),
    )?;
    svm.add_program(
        id,
        pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
        include_bytes!("programs/spl_memo-3.0.0.so"),
    )?;
    svm.add_program(
        id,
        pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
        include_bytes!("programs/spl_associated_token_account-1.1.1.so"),
    )?;
    Ok(())
}
