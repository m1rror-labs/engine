use serde_json::Value;

pub fn get_health() -> Result<Value, Value> {
    Ok(serde_json::json!("ok"))
}
