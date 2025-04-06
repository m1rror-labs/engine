use actix_ws::Session;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::RpcRequest;

pub async fn logs_unsubscribe<T: Storage + Clone + 'static>(
    req: &RpcRequest,
    mut session: Session,
    svm: &SvmEngine<T>,
) -> Result<(), String> {
    let sub_id_64 = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_u64())
    {
        Some(s) => s,
        None => {
            return Err("`params` should have at least 1 argument(s)".to_string());
        }
    };
    let sub_id = match u32::try_from(sub_id_64) {
        Ok(s) => s,
        Err(_) => {
            return Err("Invalid `sub_id` value".to_string());
        }
    };

    match svm.logs_unsubscribe(sub_id) {
        Ok(()) => {
            session
                .text(
                    serde_json::json!({
                      "jsonrpc": "2.0",
                      "id": req.id,
                      "result": true
                    })
                    .to_string(),
                )
                .await
                .map_err(|e| e.to_string())?;
        }
        Err(_) => {
            session
                .text(
                    serde_json::json!({
                      "jsonrpc": "2.0",
                      "id": req.id,
                      "result": false
                    })
                    .to_string(),
                )
                .await
                .map_err(|e| e.to_string())?;
        }
    };

    Ok(())
}
