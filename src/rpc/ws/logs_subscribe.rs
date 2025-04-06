use actix_ws::Session;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    rpc::rpc::parse_pubkey,
    storage::Storage,
};

use super::RpcRequest;

pub async fn logs_subscribe<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    mut session: Session,
    svm: &SvmEngine<T>,
) -> Result<(), String> {
    let mentions = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("mentions"))
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr,
        None => {
            println!(
                "metions params: {:?}",
                req.params
                    .as_ref()
                    .and_then(|params| params.get(0))
                    .and_then(|v| v.as_object())
            );
            return Err("`params` should have an argument with a `mentions` field".to_string());
        }
    };

    if mentions.len() != 1 {
        return Err("`mentions` must have 1 argument".to_string());
    }
    let pubkey_str = match mentions[0].as_str() {
        Some(s) => s,
        None => {
            return Err("`mentions` should be a string".to_string());
        }
    };
    let pubkey = parse_pubkey(pubkey_str).map_err(|e| e.to_string())?;

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

    let mut receiver = match svm.logs_subscribe(id, sub_id, &pubkey) {
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
        let (signature, _, transaction_meta, _) = match res {
            Some(res) => res,
            None => return Ok(()),
        };
        count = count + 1;

        session
            .text(
                serde_json::json!({
                  "jsonrpc": "2.0",
                  "method": "logsNotification",
                  "params": {
                    "result": {
                        "context": {
                          "slot": 5208469
                        },
                        "value": {
                          "signature": signature.to_string(),
                          "err": transaction_meta.err,
                          "logs": transaction_meta.log_messages,
                        }
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
