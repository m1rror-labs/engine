use actix_web::rt;
use bigdecimal::ToPrimitive;
use rdkafka::admin::{AdminClient, NewTopic};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{BaseProducer, BaseRecord};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::{accounts::DbAccount, blocks::DbBlock, transactions::DbTransactionObject};

#[derive(Clone)]
pub struct Pubsub {
    producer: Arc<Mutex<BaseProducer>>,
}

impl Pubsub {
    pub fn new(url: &str) -> Self {
        let producer = Arc::new(Mutex::new(
            ClientConfig::new()
                .set("bootstrap.servers", url)
                .create()
                .expect("Producer creation error"),
        ));

        // run a blocking task to create the topic
        let url_clone = url.to_string();
        rt::spawn(async move {
            let admin_client = ClientConfig::new()
                .set("bootstrap.servers", url_clone)
                .set("broker.address.family", "v4") // Force IPv4
                .create::<AdminClient<_>>()
                .expect("Admin client creation error");

            let geyser_topic =
                NewTopic::new("geyser", 1, rdkafka::admin::TopicReplication::Fixed(1));
            admin_client
                .create_topics(&[geyser_topic], &rdkafka::admin::AdminOptions::new())
                .await
                .expect("Failed to create topics");
        });

        Pubsub { producer }
    }

    pub fn publish_account_update(&self, account: DbAccount) {
        let producer = self.producer.lock().unwrap();
        let payload = serde_json::to_string(&PubSubAccount::from_db_account(account)).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("geyser")
                .payload(payload.as_str())
                .key("account"),
        ) {
            println!("Failed to send message to Kafka: {:?}", e);
        }
    }

    pub fn publish_accounts_update(&self, accounts: Vec<DbAccount>) {
        let producer = self.producer.lock().unwrap();
        for account in accounts {
            let payload = serde_json::to_string(&PubSubAccount::from_db_account(account)).unwrap();
            if let Err(e) = producer.send(
                BaseRecord::to("geyser")
                    .payload(payload.as_str())
                    .key("account"),
            ) {
                println!("Failed to send message to Kafka: {:?}", e);
            }
        }
    }

    pub fn publish_transaction(&self, transaction: DbTransactionObject) {
        let producer = self.producer.lock().unwrap();
        let payload = serde_json::to_string(&PubSubTransactionObject::from_db_transaction_object(
            transaction,
        ))
        .unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("geyser")
                .payload(payload.as_str())
                .key("transaction"),
        ) {
            println!("Failed to send message to Kafka: {:?}", e);
        }
    }

    pub fn publish_block(&self, block: DbBlock) {
        let producer = self.producer.lock().unwrap();
        let payload = serde_json::to_string(&PubSubBlock::from_db_block(block)).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("geyser")
                .payload(payload.as_str())
                .key("block"),
        ) {
            println!("Failed to send message to Kafka: {:?}", e);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubAccount {
    pub id: Uuid,
    pub address: String,
    pub lamports: u128,
    pub data: Vec<u8>,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u128,
    pub label: Option<String>,
    pub blockchain: Uuid,
}

impl PubSubAccount {
    pub fn from_db_account(db_account: DbAccount) -> Self {
        PubSubAccount {
            id: db_account.id,
            address: db_account.address,
            lamports: db_account.lamports.to_u128().unwrap(),
            data: db_account.data,
            owner: db_account.owner,
            executable: db_account.executable,
            rent_epoch: db_account.rent_epoch.to_u128().unwrap(),
            label: db_account.label,
            blockchain: db_account.blockchain,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubBlock {
    pub id: Uuid,
    pub blockchain: Uuid,
    pub blockhash: Vec<u8>,
    pub previous_blockhash: Vec<u8>,
    pub parent_slot: u128,
    pub block_height: u128,
    pub slot: u128,
}

impl PubSubBlock {
    pub fn from_db_block(db_block: DbBlock) -> Self {
        PubSubBlock {
            id: db_block.id,
            blockchain: db_block.blockchain,
            blockhash: db_block.blockhash,
            previous_blockhash: db_block.previous_blockhash,
            parent_slot: db_block.parent_slot.to_u128().unwrap(),
            block_height: db_block.block_height.to_u128().unwrap(),
            slot: db_block.slot.to_u128().unwrap(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct PubSubTransactionObject {
    pub transaction: PubSubTransaction,
    pub meta: PubSubTransactionMeta,
    pub account_keys: Vec<PubSubTransactionAccountKey>,
    pub instructions: Vec<PubSubTransactionInstruction>,
    pub log_messages: Vec<PubSubTransactionLogMessage>,
    pub signatures: Vec<PubSubTransactionSignature>,
    pub token_balances: Vec<PubSubTransactionTokenBalance>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransaction {
    pub id: Uuid,
    pub signature: String,
    pub version: String,
    pub recent_blockhash: Vec<u8>,
    pub slot: u128,
    pub blockchain: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransactionMeta {
    pub id: Uuid,
    pub transaction_signature: String,
    pub err: Option<String>,
    pub compute_units_consumed: u128,
    pub fee: u128,
    pub pre_balances: Vec<i64>,
    pub post_balances: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransactionAccountKey {
    pub id: Uuid,
    pub transaction_signature: String,
    pub account: String,
    pub signer: bool,
    pub writable: bool,
    pub index: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransactionInstruction {
    pub id: Uuid,
    pub transaction_signature: String,
    pub accounts: Vec<i16>,
    pub data: Vec<u8>,
    pub program_id: String,
    pub stack_height: i16,
    pub inner: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransactionLogMessage {
    pub id: Uuid,
    pub transaction_signature: String,
    pub log: String,
    pub index: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransactionSignature {
    pub id: Uuid,
    pub transaction_signature: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PubSubTransactionTokenBalance {
    pub id: Uuid,
    pub account_index: i16,
    pub transaction_signature: String,
    pub mint: String,
    pub owner: String,
    pub program_id: String,
    pub amount: u128,
    pub decimals: i16,
    pub pre_transaction: bool,
}

impl PubSubTransactionObject {
    pub fn from_db_transaction_object(db_transaction_object: DbTransactionObject) -> Self {
        PubSubTransactionObject {
            transaction: PubSubTransaction {
                id: db_transaction_object.transaction.id,
                signature: db_transaction_object.transaction.signature,
                version: db_transaction_object.transaction.version,
                recent_blockhash: db_transaction_object.transaction.recent_blockhash,
                slot: db_transaction_object.transaction.slot.to_u128().unwrap(),
                blockchain: db_transaction_object.transaction.blockchain,
            },
            meta: PubSubTransactionMeta {
                id: db_transaction_object.meta.id,
                transaction_signature: db_transaction_object.meta.transaction_signature,
                err: db_transaction_object.meta.err,
                compute_units_consumed: db_transaction_object
                    .meta
                    .compute_units_consumed
                    .to_u128()
                    .unwrap(),
                fee: db_transaction_object.meta.fee.to_u128().unwrap(),
                pre_balances: db_transaction_object
                    .meta
                    .pre_balances
                    .iter()
                    .map(|x| x.to_i64().unwrap())
                    .collect(),
                post_balances: db_transaction_object
                    .meta
                    .post_balances
                    .iter()
                    .map(|x| x.to_i64().unwrap())
                    .collect(),
            },
            account_keys: db_transaction_object
                .account_keys
                .iter()
                .map(|x| PubSubTransactionAccountKey {
                    id: x.id,
                    transaction_signature: x.transaction_signature.clone(),
                    account: x.account.clone(),
                    signer: x.signer,
                    writable: x.writable,
                    index: x.index,
                })
                .collect(),
            instructions: db_transaction_object
                .instructions
                .iter()
                .map(|x| PubSubTransactionInstruction {
                    id: x.id,
                    transaction_signature: x.transaction_signature.clone(),
                    accounts: x.accounts.clone(),
                    data: x.data.clone(),
                    program_id: x.program_id.clone(),
                    stack_height: x.stack_height,
                    inner: x.inner,
                })
                .collect(),
            log_messages: db_transaction_object
                .log_messages
                .iter()
                .map(|x| PubSubTransactionLogMessage {
                    id: x.id,
                    transaction_signature: x.transaction_signature.clone(),
                    log: x.log.clone(),
                    index: x.index,
                })
                .collect(),
            signatures: db_transaction_object
                .signatures
                .iter()
                .map(|x| PubSubTransactionSignature {
                    id: x.id,
                    transaction_signature: x.transaction_signature.clone(),
                    signature: x.signature.clone(),
                })
                .collect(),
            token_balances: db_transaction_object
                .token_balances
                .iter()
                .map(|x| PubSubTransactionTokenBalance {
                    id: x.id,
                    account_index: x.account_index,
                    transaction_signature: x.transaction_signature.clone(),
                    mint: x.mint.clone(),
                    owner: x.owner.clone(),
                    program_id: x.program_id.clone(),
                    amount: x.amount.to_u128().unwrap(),
                    decimals: x.decimals,
                    pre_transaction: x.pre_transaction,
                })
                .collect(),
        }
    }
}
