use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Transaction {
    pub id: Option<String>,
    pub date: i64,
    pub details: String,
    pub amount: Decimal,
    pub category: Option<String>,
}

impl Transaction {
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.input(&self.date.to_be_bytes());
        hasher.input(self.details.as_bytes());
        hasher.input(self.amount.to_string().as_bytes());
        hasher.result_str()
    }
}
