use crate::transaction::Transaction;
use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Statement {
    pub categories: Vec<CategoryOverview>,
    pub total_credits: Decimal,
    pub total_debits: Decimal,
    pub credits: Vec<Transaction>,
    pub debits: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct CategoryOverview {
    pub name: String,
    pub count: i64,
    pub credits: Decimal,
    pub debits: Decimal,
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

    pub fn get_credits_for_category(&self, name: &str) -> Vec<&Transaction> {
        self.credits.iter().filter(|c| c.category == name).collect()
    }

    pub fn get_debits_for_category(&self, name: &str) -> Vec<&Transaction> {
        self.debits.iter().filter(|c| c.category == name).collect()
    }

    pub fn calculate_category_overview(&mut self) -> &Vec<CategoryOverview> {
        let mut categories: Vec<CategoryOverview> = Vec::new();

        let credit_categories: Vec<&String> = self.credits.iter().map(|t| &t.category).collect();
        let debit_categories: Vec<&String> = self.debits.iter().map(|t| &t.category).collect();

        for category in credit_categories {
            if categories.iter().find(|c| &c.name == category).is_none() {
                categories.push(CategoryOverview {
                    name: category.to_string(),
                    count: 0i64,
                    credits: Decimal::new(0, 2),
                    debits: Decimal::new(0, 2),
                })
            }
        }

        for category in debit_categories {
            if categories.iter().find(|c| &c.name == category).is_none() {
                categories.push(CategoryOverview {
                    name: category.to_string(),
                    count: 0i64,
                    credits: Decimal::new(0, 2),
                    debits: Decimal::new(0, 2),
                })
            }
        }

        for transaction in self.credits.clone() {
            if let Some(mut category) = categories
                .iter_mut()
                .find(|c| c.name == transaction.category)
            {
                category.count += 1;
                category.credits += transaction.amount;
            }
        }

        for transaction in self.debits.clone() {
            if let Some(mut category) = categories
                .iter_mut()
                .find(|c| c.name == transaction.category)
            {
                category.count += 1;
                category.debits += transaction.amount;
            }
        }

        self.categories = categories;
        &self.categories
    }
}
