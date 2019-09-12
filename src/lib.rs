pub mod parser;

use rust_decimal::Decimal;

#[derive(Clone, Default, Debug)]
pub struct Statement {
    name: String,
    date: i64,
    total_credits: Decimal,
    total_debits: Decimal,
    opening_balance: Decimal,
    closing_balance: Decimal,
    credits: Vec<Transaction>,
    debits: Vec<Transaction>,
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

#[derive(Clone, Debug)]
pub struct Transaction {
    date: i64,
    details: String,
    amount: Decimal,
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_pdf() {
        let file = std::fs::File::open("samples/september19.pdf").unwrap();
        let mut parser = crate::parser::Parser::new();

        match parser.parse(file) {
            Ok(statement) => {
                // println!("{:#?}", statement);
                println!(
                    "\n\n\nFound {} transactions",
                    statement.credits.len() + statement.debits.len()
                );
                println!("Statement reconciled: {}", statement.validate());
            }

            Err(e) => println!("Error: {:#?}", e),
        }
    }
}
