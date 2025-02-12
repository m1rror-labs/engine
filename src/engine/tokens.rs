use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: u64,
    pub decimals: u8,
    pub ui_amount: f64,
    pub ui_amount_string: String,
}
