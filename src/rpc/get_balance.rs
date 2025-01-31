use super::rpc::{RpcRequest, RpcResponse};

pub fn get_balance(req: RpcRequest) -> RpcResponse {
    RpcResponse {
        jsonrpc: req.jsonrpc,
        id: req.id,
        result: Some(serde_json::json!({
            "balance": 1000,
        })),
        error: None,
    }
}
