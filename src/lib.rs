pub mod parser;

#[derive(Clone, Default, Debug)]
pub struct Statement {
    name: String,
    date: i64,
    opening_balance: f32,
    closing_balance: f32,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Default, Debug)]
pub struct Transaction {
    date: i64,
    details: String,
    amount: f64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_pdf() {
        let file = std::fs::File::open("samples/september19.pdf").unwrap();
        let mut parser = crate::parser::Parser::new();

        match parser.parse(file) {
            Ok(statement) => {
                println!("{:#?}", statement);
                println!("\n\n\nFound {} transactions", statement.transactions.len());
                let total = statement
                    .transactions
                    .iter()
                    .fold(0f64, |sum, t| sum + t.amount);
                println!("Total spend: {:.2}\n", total);
                println!("First Transaction: {:#?}", statement.transactions[0]);
            }
            Err(e) => println!("Error: {:#?}", e),
        }
    }
}
