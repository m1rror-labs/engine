use actix_ws::Session;
use solana_banks_interface::TransactionConfirmationStatus;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    rpc::rpc::parse_signature,
    storage::Storage,
};

use super::RpcRequest;

pub async fn signature_subscribe<T: Storage + Clone>(
    id: Uuid,
    req: &RpcRequest,
    mut session: Session,
    svm: &SvmEngine<T>,
) -> Result<(), String> {
    let sig_str = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_str())
    {
        Some(s) => s,
        None => {
            return Err("`params` should have at least 1 argument(s)".to_string());
        }
    };
    let commitment = match req
        .params
        .as_ref()
        .and_then(|params| params.get(1))
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("commitment"))
        .and_then(|v| v.as_str())
    {
        Some(c) => c.to_string(),
        None => {
            return Err("`commitment` parameter is missing or invalid".to_string());
        }
    };
    let confirmation = match commitment.as_str() {
        "finalized" => TransactionConfirmationStatus::Finalized,
        "confirmed" => TransactionConfirmationStatus::Confirmed,
        "processed" => TransactionConfirmationStatus::Processed,
        _ => return Err("Invalid `commitment` value".to_string()),
    };

    let sub_id = rand::random::<u64>();
    session
        .text(
            serde_json::json!({
              "jsonrpc": "2.0",
              "id": 123,
              "result": sub_id
            })
            .to_string(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let signature = parse_signature(sig_str).map_err(|e| e.to_string())?;
    match svm.signature_subscribe(id, &signature, confirmation).await {
        Ok(_) => {
            println!("Signature subscribed: {:?}", signature);
            session
                .text(
                    serde_json::json!({
                      "jsonrpc": "2.0",
                      "method": "signatureNotification",
                      "params": {
                        "result": {
                          "context": {
                            "slot": 5207624
                          },
                          "value": {
                            "err": null
                          }
                        },
                        "subscription": sub_id
                      }
                    })
                    .to_string(),
                )
                .await
                .map_err(|e| e.to_string())?
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}
