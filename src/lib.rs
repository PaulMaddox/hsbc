pub mod parser;

#[derive(Clone, Default, Debug)]
pub struct Statement {
    name: String,
    date: u32,
    opening_balance: f32,
    closing_balance: f32,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Default, Debug)]
pub struct Transaction {
    date: u32,
    details: String,
    amount: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_pdf() {
        let file = std::fs::File::open("samples/july19.pdf").unwrap();
        let mut parser = crate::parser::Parser::new();

        match parser.parse(file) {
            Ok(statement) => {
                println!(
                    "Statement contains {} transactions",
                    statement.transactions.len()
                );
                // println!("{:#?}", statement.transactions);
            }
            Err(e) => println!("Error: {:#?}", e),
        }
    }
}
