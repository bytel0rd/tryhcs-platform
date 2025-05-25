use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankInfo {
    pub name: String,
pub code: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccountInfo {
    pub name: String,
pub account_nmuber: String,
pub bank_code: String,
}
