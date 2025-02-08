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
    pub lamports: i64,
    pub data: Vec<u8>,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: i64,
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
            lamports: account.lamports as i64,
            data: account.data.clone(),
            owner: account.owner.to_string(),
            executable: account.executable,
            rent_epoch: account.rent_epoch as i64,
            label,
            blockchain,
        }
    }

    pub fn into_account(self) -> Account {
        Account {
            //TODO: Will the i64 to u64 conversion cause issues?
            lamports: self.lamports.try_into().unwrap(),
            data: self.data,
            owner: Pubkey::from_str(&self.owner).unwrap(),
            executable: self.executable,
            rent_epoch: self.rent_epoch.try_into().unwrap(),
        }
    }
}
