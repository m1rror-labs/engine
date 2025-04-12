use actix_ws::Session;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::RpcRequest;

pub async fn slot_subscribe<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    mut session: Session,
    svm: &SvmEngine<T>,
) -> Result<(), String> {
    let sub_id = rand::random::<u32>();
    session
        .text(
            serde_json::json!({
              "jsonrpc": "2.0",
              "id": req.id,
              "result": sub_id
            })
            .to_string(),
        )
        .await
        .map_err(|e| e.to_string())?;
    println!("slot subscribe");

    let mut receiver = match svm.slot_subscribe(id, sub_id) {
        Ok(rec) => rec,
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(e);
        }
    };
    let mut count = 1;

    loop {
        let res = match receiver.recv().await {
            Some(res) => res,
            None => return Ok(()),
        };
        let (parent, root, slot) = match res {
            Some(res) => res,
            None => return Ok(()),
        };
        count = count + 1;
        println!(
            "parent: {}, root: {}, slot: {}, count: {}, current time: {}",
            parent,
            root,
            slot,
            count,
            chrono::Utc::now().to_rfc3339()
        );

        session
            .text(
                serde_json::json!({
                  "jsonrpc": "2.0",
                  "method": "slotNotification",
                  "params": {
                    "result": {
                      "parent": parent,
                      "root": root,
                      "slot": slot
                    },
                    "subscription": sub_id
                  }
                })
                .to_string(),
            )
            .await
            .map_err(|e| e.to_string())?;
    }
}
