use std::sync::Arc;

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey};

#[derive(Clone)]
pub struct Rpc {
    client: Arc<RpcClient>,
}

impl Rpc {
    pub fn new(url: String) -> Self {
        let client = Arc::new(RpcClient::new(url));
        Self { client }
    }

    pub async fn get_account(&self, pubkey: &Pubkey) -> Result<Option<Account>, String> {
        let account = self
            .client
            .get_account_with_commitment(pubkey, CommitmentConfig::confirmed())
            .await
            .map_err(|e| e.to_string())?;
        Ok(account.value)
    }

    pub async fn get_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<Option<Account>>, String> {
        let accounts = self
            .client
            .get_multiple_accounts_with_commitment(pubkeys, CommitmentConfig::confirmed())
            .await
            .map_err(|e| e.to_string())?;
        Ok(accounts.value)
    }
}
