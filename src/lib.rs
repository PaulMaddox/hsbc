mod parser;
mod pdf;

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
        let data = std::fs::read("../statement/august19.pdf").unwrap();
        match super::parser::parse_streams(&data) {
            Ok((_, streams)) => {
                for (i, stream) in streams.iter().enumerate() {
                    println!("Stream {} (length: {} bytes)", i, stream.bytes.len());

                    let mut decompressed: Vec<u8> = Vec::new();
                    match stream.decompress(&mut decompressed) {
                        Ok(_) => println!("{}", String::from_utf8_lossy(&decompressed)),
                        Err(e) => println!("{}", e),
                    }
                }
            }
            Err(e) => {
                println!("{:#?}", e);
            }
        }
    }
}
