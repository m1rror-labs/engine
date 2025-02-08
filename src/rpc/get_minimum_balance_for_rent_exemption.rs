use serde_json::Value;
use solana_sdk::message::AddressLoader;

use crate::{engine::SVM, storage::Storage};

use super::rpc::{Dependencies, RpcRequest};

pub fn get_minimum_balance_for_rent_exemption<T: Storage + AddressLoader>(
    req: &RpcRequest,
    deps: &Dependencies<T>,
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

    let svm = deps.svm.write().unwrap();

    let balance = svm.minimum_balance_for_rent_exemption(size);
    Ok(Value::Number(balance.into()))
}
