use solana_program::pubkey;
use uuid::Uuid;

use crate::storage::Storage;

use super::{SvmEngine, SVM};

pub fn load_spl_programs<T: Storage + Clone + 'static>(svm: &SvmEngine<T>, id: Uuid) -> Result<(), String> {
    svm.add_program(
        id,
        pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        include_bytes!("programs/spl_token-3.5.0.so"),
    )?;
    svm.add_program(
        id,
        pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
        include_bytes!("programs/spl_token_2022-1.0.0.so"),
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
