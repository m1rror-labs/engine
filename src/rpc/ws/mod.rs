use crate::{engine::SvmEngine, storage::Storage};
use actix_ws::Session;
use futures::TryFutureExt;
use serde::Deserialize;
use signature_subscribe::signature_subscribe;
use uuid::Uuid;
pub mod signature_subscribe;

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
        RpcMethod::LogsSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("LogsSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
        RpcMethod::LogsUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("LogsUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
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
            signature_subscribe(id, &req, session.clone(), svm).await?
        }
        RpcMethod::SlotSubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("SlotSubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
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
        RpcMethod::SlotUnsubscribe => {
            session
                .close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Normal,
                    description: Some("SlotUnsubscribe not implemented".into()),
                }))
                .map_err(|e| e.to_string())
                .await?;
        }
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
