use actix_web::rt;
use rdkafka::admin::{AdminClient, NewTopic};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{BaseProducer, BaseRecord};
use std::sync::{Arc, Mutex};

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
                .create::<AdminClient<_>>()
                .expect("Admin client creation error");

            let geyser_topic =
                NewTopic::new("geyser", 1, rdkafka::admin::TopicReplication::Variable);
            admin_client
                .create_topics(&[geyser_topic], &rdkafka::admin::AdminOptions::new())
                .await
                .expect("Failed to create topics");
        });

        Pubsub { producer }
    }

    pub fn publish_account_update(&self, account: DbAccount) {
        let producer = self.producer.lock().unwrap();
        let payload = serde_json::to_string(&account).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("accounts")
                .payload(payload.as_str())
                .key("account"),
        ) {
            println!("Failed to send message to Kafka: {:?}", e);
        }
    }

    pub fn publish_accounts_update(&self, accounts: Vec<DbAccount>) {
        let producer = self.producer.lock().unwrap();
        for account in accounts {
            let payload = serde_json::to_string(&account).unwrap();
            if let Err(e) = producer.send(
                BaseRecord::to("accounts")
                    .payload(payload.as_str())
                    .key("account"),
            ) {
                println!("Failed to send message to Kafka: {:?}", e);
            }
        }
    }

    pub fn publish_transaction(&self, transaction: DbTransactionObject) {
        let producer = self.producer.lock().unwrap();
        let payload = serde_json::to_string(&transaction).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("transactions")
                .payload(payload.as_str())
                .key("transaction"),
        ) {
            println!("Failed to send message to Kafka: {:?}", e);
        }
    }

    pub fn publish_block(&self, block: DbBlock) {
        let producer = self.producer.lock().unwrap();
        let payload = serde_json::to_string(&block).unwrap();
        if let Err(e) = producer.send(
            BaseRecord::to("blocks")
                .payload(payload.as_str())
                .key("block"),
        ) {
            println!("Failed to send message to Kafka: {:?}", e);
        }
    }
}
