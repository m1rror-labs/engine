use crate::{engine::SvmEngine, storage::Storage};
use actix_ws::Session;
use futures::TryFutureExt;
use logs_subscribe::logs_subscribe;
use logs_unsubscribe::logs_unsubscribe;
use serde::Deserialize;
use signature_subscribe::signature_subscribe;
use slot_subscribe::slot_subscribe;
use slot_unsubscribe::slot_unsubscribe;
use uuid::Uuid;
pub mod logs_subscribe;
pub mod logs_unsubscribe;
pub mod signature_subscribe;
pub mod slot_subscribe;
pub mod slot_unsubscribe;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum RpcMethod {
    AccountSubscribe,
    AccountUnsubscribe,
    BlockSubscribe,
    BlockUnsubscribe,
    LogsSubscribe,
    LogsUnsubscribe,
    ProgramSubscribe,
    ProgramUnsubscribe,
    RootSubscribe,
    RootUnsubscribe,
    SignatureSubscribe,
    SignatureUnsubscribe,
    SlotSubscribe,
    SlotsUpdatesSubscribe,
    SlotsUpdatesUnsubscribe,
    SlotUnsubscribe,
    VoteSubscribe,
    VoteUnsubscribe,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: RpcMethod,
    pub params: Option<serde_json::Value>,
}

pub async fn handle_ws_request<T: Storage + Clone + 'static>(
    id: Uuid,
    msg: &str,
    session: Session,
    svm: &SvmEngine<T>,
) -> Result<(), String> {
    let req: RpcRequest = serde_json::from_str(msg).map_err(|e| e.to_string())?;

    match req.method {
        RpcMethod::AccountSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("AccountSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::AccountUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("AccountUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::BlockSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("BlockSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::BlockUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("BlockUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::LogsSubscribe => logs_subscribe(id, &req, session, svm).await?,
        RpcMethod::LogsUnsubscribe => logs_unsubscribe(&req, session, svm).await?,
        RpcMethod::ProgramSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("ProgramSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::ProgramUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("ProgramUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::RootSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("RootSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::RootUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("RootUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::SignatureSubscribe => signature_subscribe(id, &req, session, svm).await?,
        RpcMethod::SignatureUnsubscribe => {
            println!("SignatureUnsubscribe");
            signature_subscribe(id, &req, session.clone(), svm).await? //TODO: This should be its own function
        }
        RpcMethod::SlotSubscribe => slot_subscribe(id, &req, session, svm).await?,
        RpcMethod::SlotsUpdatesSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("SlotsUpdatesSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::SlotsUpdatesUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("SlotsUpdatesUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::SlotUnsubscribe => slot_unsubscribe(&req, session, svm).await?,
        RpcMethod::VoteSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("VoteSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::VoteUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("VoteUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
    };
    Ok(())
}
