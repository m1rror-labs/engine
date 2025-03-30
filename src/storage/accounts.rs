use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::prelude::*;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbAccount {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub address: String,
    pub lamports: BigDecimal,
    pub data: Vec<u8>,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: BigDecimal,
    pub label: Option<String>,
    pub blockchain: Uuid,
}

impl DbAccount {
    pub fn from_account(
        pubkey: &Pubkey,
        account: &Account,
        label: Option<String>,
        blockchain: Uuid,
    ) -> Self {
        DbAccount {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_utc(),
            address: pubkey.to_string(),
            lamports: account.lamports.into(),
            data: account.data.clone(),
            owner: account.owner.to_string(),
            executable: account.executable,
            rent_epoch: account.rent_epoch.into(),
            label,
            blockchain,
        }
    }

    pub fn into_account(self) -> Account {
        Account {
            lamports: self.lamports.to_u64().unwrap(),
            data: self.data,
            owner: Pubkey::from_str(&self.owner).unwrap(),
            executable: self.executable,
            rent_epoch: self.rent_epoch.to_u64().unwrap(),
        }
    }
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::blockchain_config_accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbConfigAccount {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub address: String,
    pub lamports: BigDecimal,
    pub data: Vec<u8>,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: BigDecimal,
    pub label: Option<String>,
    pub config: Uuid,
}

impl DbConfigAccount {
    pub fn from_account(
        pubkey: &Pubkey,
        account: &Account,
        label: Option<String>,
        config: Uuid,
    ) -> Self {
        DbConfigAccount {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_utc(),
            address: pubkey.to_string(),
            lamports: account.lamports.into(),
            data: account.data.clone(),
            owner: account.owner.to_string(),
            executable: account.executable,
            rent_epoch: account.rent_epoch.into(),
            label,
            config,
        }
    }

    pub fn into_account(self) -> Account {
        Account {
            lamports: self.lamports.to_u64().unwrap(),
            data: self.data,
            owner: Pubkey::from_str(&self.owner).unwrap(),
            executable: self.executable,
            rent_epoch: self.rent_epoch.to_u64().unwrap(),
        }
    }
}
