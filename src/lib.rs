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
        let file = std::fs::File::open("samples/july19.pdf").unwrap();
        let mut parser = crate::parser::Parser::new();

        match parser.parse(file) {
            Ok(statement) => println!("{:#?}", statement),
            Err(e) => println!("Error: {:#?}", e),
        }
    }
}
