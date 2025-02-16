use serde_json::Value;
use solana_banks_interface::{TransactionConfirmationStatus, TransactionStatus};
use solana_sdk::transaction::Transaction;
use uuid::Uuid;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::{parse_signature, RpcRequest};

pub fn get_signature_statuses<T: Storage + Clone + 'static>(
    id: Uuid,
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    println!("get_signature_statuses");
    println!("{:?}", req);
    let sig_raw_arr = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|arr| arr.as_array()) //TODO: This needs to handle multiple signatures
    {
        Some(s) => s,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };
    let sig_arr = sig_raw_arr
        .iter()
        .map(|sig| sig.as_str().unwrap())
        .collect::<Vec<&str>>();

    let sigs = sig_arr
        .iter()
        .map(|sig_str| parse_signature(sig_str))
        .collect::<Result<Vec<solana_sdk::signature::Signature>, Value>>()?;

    let txs: Vec<Option<(Transaction, TransactionStatus)>> =
        sigs.iter()
            .map(|sig| svm.get_transaction(id, &sig))
            .collect::<Result<
                Vec<Option<(solana_sdk::transaction::Transaction, TransactionStatus)>>,
                String,
            >>()?;

    Ok(serde_json::json!({
        "context": { "slot": 341197053,"apiVersion":"1.18.1" },
        "value": txs
        .iter()
        .map(|tx| match tx {
            Some((_, status)) => {
                let status_value = match status.err.clone() {
                    Some(err) => {
                        serde_json::json!({
                            "Err": err,
                        })
                    }
                    None => {
                        serde_json::json!({
                                "Ok": null
                        })
                    }
                };
                serde_json::json!({
                    "slot": status.slot,
                    "confirmations": null,
                    "err": status.err,
                    "status": status_value,
                    "confirmationStatus": match status.clone().confirmation_status {
                        Some(status) => match status {
                            TransactionConfirmationStatus::Finalized => "finalized",
                            TransactionConfirmationStatus::Confirmed => "confirmed",
                            TransactionConfirmationStatus::Processed => "processed",
                        },
                        None => "processed",
                    }
                })
            }
            None => serde_json::json!(null),
        })
        .collect::<Vec<Value>>(),
    }))

    // match svm.get_transaction(id, &signature) {
    //     Ok(transaction) => match transaction {
    //         Some((_, status)) => {
    //             let status_value = match status.err.clone() {
    //                 Some(err) => {
    //                     serde_json::json!({
    //                         "err": err,
    //                     })
    //                 }
    //                 None => {
    //                     serde_json::json!({
    //                             "Ok": null
    //                     })
    //                 }
    //             };
    //             Ok(serde_json::json!({
    //                 "context": { "slot": status.slot,"apiVersion":"1.18.1" },
    //                 "value": [
    //                     {
    //                       "slot": status.slot,
    //                       "confirmations": null,
    //                       "err": status.err,
    //                       "status": status_value,
    //                       "confirmationStatus": match status.confirmation_status {
    //                           Some(status) => match status {
    //                               TransactionConfirmationStatus::Finalized => "finalized",
    //                               TransactionConfirmationStatus::Confirmed => "confirmed",
    //                               TransactionConfirmationStatus::Processed => "processed",
    //                           },
    //                           None => "processed",
    //                       }
    //                     },
    //                     null
    //                   ]
    //             }))
    //         }
    //         None => Ok(serde_json::json!({
    //             "context": { "slot": 341197053,"apiVersion":"1.18.1" },
    //             "value": null,
    //         })),
    //     },
    //     Err(e) => Err(serde_json::json!({
    //         "code": -32002,
    //         "message": e,
    //     })),
    // }
}
