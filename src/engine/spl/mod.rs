use solana_program::pubkey;
use solana_sdk::{account::Account, pubkey::Pubkey};

use crate::storage::Storage;

use super::{SvmEngine, SVM};

pub fn generate_spl_programs<T: Storage + Clone + 'static>(
    svm: &SvmEngine<T>,
) -> Vec<(Pubkey, Account)> {
    vec![
        svm.add_program(
            pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            include_bytes!("programs/spl_token-3.5.0.so"),
        ),
        svm.add_program(
            pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
            include_bytes!("programs/spl_token_2022.so"),
        ),
        svm.add_program(
            pubkey!("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo"),
            include_bytes!("programs/spl_memo-1.0.0.so"),
        ),
        svm.add_program(
            pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
            include_bytes!("programs/spl_memo-3.0.0.so"),
        ),
        svm.add_program(
            pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"),
            include_bytes!("programs/spl_associated_token_account-1.1.1.so"),
        ),
        svm.add_program(
            pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
            include_bytes!("programs/metaplex_metadata_program.so"),
        ),
    ]
}
