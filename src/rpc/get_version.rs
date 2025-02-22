use serde_json::Value;

pub fn get_version() -> Result<Value, Value> {
    Ok(serde_json::json!( { "feature-set": 2891131721u32, "solana-core": "2.1.13" }))
}
