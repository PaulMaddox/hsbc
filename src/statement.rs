use crate::transaction::Transaction;
use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Statement {
    pub total_credits: Decimal,
    pub total_debits: Decimal,
    pub credits: Vec<Transaction>,
    pub debits: Vec<Transaction>,
}

impl Statement {
    // Validates that the sum of all credits/debits matches the statement summary
    pub fn validate(&self) -> bool {
        let total_credits = self
            .credits
            .iter()
            .fold(Decimal::new(0, 2), |sum, t| sum + t.amount);
        let total_debits = self
            .debits
            .iter()
            .fold(Decimal::new(0, 2), |sum, t| sum + t.amount);
        total_credits == self.total_credits && total_debits == self.total_debits
    }
}
