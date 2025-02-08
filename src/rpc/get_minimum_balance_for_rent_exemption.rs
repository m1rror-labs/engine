use serde_json::Value;

use crate::{
    engine::{SvmEngine, SVM},
    storage::Storage,
};

use super::rpc::RpcRequest;

pub fn get_minimum_balance_for_rent_exemption<T: Storage + Clone>(
    req: &RpcRequest,
    svm: &SvmEngine<T>,
) -> Result<Value, Value> {
    let size = match req
        .params
        .as_ref()
        .and_then(|params| params.get(0))
        .and_then(|v| v.as_u64())
    {
        Some(s) => s as usize,
        None => {
            return Err(serde_json::json!({
                "code": -32602,
                "message": "`params` should have at least 1 argument(s)"
            }));
        }
    };

    let balance = svm.minimum_balance_for_rent_exemption(size);
    Ok(Value::Number(balance.into()))
}
