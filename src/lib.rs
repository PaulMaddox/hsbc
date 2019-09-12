pub mod category;
pub mod parser;
pub mod statement;
pub mod transaction;

#[cfg(test)]
mod tests {
    #[test]
    fn parses_pdf() {
        let file = std::fs::File::open("samples/september19.pdf").unwrap();
        let mut parser = crate::parser::Parser::new(Vec::new());

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
