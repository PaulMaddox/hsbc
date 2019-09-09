pub mod error;
pub mod parser;
pub mod pdf;

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
        let file = std::fs::File::open("samples/august19.pdf").unwrap();
        let mut parser = crate::parser::Parser::new();

        let statement = parser.parse(file).unwrap();
        println!("{:#?}", statement);
    }
}
