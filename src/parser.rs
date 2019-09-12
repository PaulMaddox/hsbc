use super::statement::Statement;
use super::transaction::Transaction;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use flate2::read::DeflateDecoder;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;
use rust_decimal::Decimal;
use std::io::prelude::*;
use std::str::*;

#[derive(Clone, Default, Debug)]
pub struct Parser {
    compressed_streams: Vec<Stream>,
    decompressed_streams: Vec<Stream>,
    transactions: Vec<Transaction>,
    statement: Statement,
}

#[derive(Clone, Default, Debug)]
struct Stream {
    bytes: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            compressed_streams: Vec::new(),
            decompressed_streams: Vec::new(),
            transactions: Vec::new(),
            statement: Statement::default(),
        }
    }

    pub fn parse<R>(&mut self, mut rdr: R) -> Result<Statement, Box<dyn std::error::Error>>
    where
        R: Read,
    {
        // Create a new statement object to be appended to and returned
        let mut statement = Statement::default();

        // Read the binary PDF data from the reader
        let mut data = Vec::new();
        rdr.read_to_end(&mut data)?;
        // First parse the object streams from the PDF
        let (_, streams) = Parser::parse_streams(&data).unwrap();
        self.compressed_streams = streams;

        // Some may be compressed, so decompress them
        for stream in self.compressed_streams.iter() {
            let mut decompressed: Vec<u8> = Vec::new();
            let mut compressed = stream.bytes.to_vec();
            compressed.drain(0..2);
            let mut decoder = DeflateDecoder::new(compressed.as_slice());
            if decoder.read_to_end(&mut decompressed).is_ok() {
                self.decompressed_streams.push(Stream {
                    bytes: decompressed,
                });
            } else {
                self.decompressed_streams.push(Stream {
                    bytes: stream.bytes.to_vec(),
                });
            }
        }

        // Now parse out the transactions from the decompressed streams
        for (i, stream) in self.decompressed_streams.iter().enumerate() {
            Parser::search_for_statement_summary(&stream.bytes, &mut statement);
            Parser::search_for_transactions(&stream.bytes, &mut statement);
        }

        Ok(statement)
    }

    fn search_for_statement_summary(input: &[u8], statement: &mut Statement) {
        let mut bytes = input.to_vec();
        loop {
            if bytes.is_empty() {
                break;
            }

            match Parser::parse_statement_summary(&bytes) {
                Ok((remaining, (total_credits, total_debits))) => {
                    statement.total_credits = total_credits;
                    statement.total_debits = total_debits;
                    let summary_length = bytes.len() - remaining.len();
                    bytes.drain(0..summary_length);
                    continue;
                }
                Err(nom::Err::Error(_)) => {
                    // This starting offset was not the beginning of a transaction
                    // so take a single byte away, and try again
                    bytes.drain(0..1);
                }
                Err(nom::Err::Failure(e)) => {
                    println!("FAILURE: {:#?}", e.1);
                    break;
                }
                Err(_) => {
                    println!("UNKNOWN: unknown error");
                    break;
                }
            }
        }
    }

    fn search_for_transactions(input: &[u8], statement: &mut Statement) {
        let mut bytes = input.to_vec();

        loop {
            // Nothing left to parse - so stop it
            if bytes.is_empty() {
                break;
            }

            match Parser::parse_transaction(&bytes) {
                Ok((remaining, (is_credit, transaction))) => {
                    if is_credit {
                        statement.credits.push(transaction);
                    } else {
                        statement.debits.push(transaction);
                    }
                    let statement_length = bytes.len() - remaining.len();
                    bytes.drain(0..statement_length);
                    continue;
                }
                Err(nom::Err::Error(_)) => {
                    // This starting offset was not the beginning of a transaction
                    // so take a single byte away, and try again
                    bytes.drain(0..1);
                }
                Err(nom::Err::Failure(e)) => {
                    println!("FAILURE: {:#?}", e.1);
                    break;
                }
                Err(_) => {
                    println!("UNKNOWN: unknown error");
                    break;
                }
            }
        }
    }

    fn parse_statement_summary(input: &[u8]) -> IResult<&[u8], (Decimal, Decimal)> {
        let (input, _) = take_until("(")(input)?;
        let (input, _) = tag("( Payments/Credits)")(input)?;
        let (input, total_credits) = Parser::take_transaction_amnt(input)?;

        let (input, _) = take_until("(")(input)?;
        let (input, _) = tag("( New charges/debits)")(input)?;
        let (input, total_debits) = Parser::take_transaction_amnt(input)?;

        Ok((input, (total_credits, total_debits)))
    }
    fn parse_transaction(input: &[u8]) -> IResult<&[u8], (bool, Transaction)> {
        // (24AUG)Tj 1 0 0 1 109.9 324.4 Tm
        // (22AUG)Tj 1 0 0 1 149.5 324.4 Tm
        // (NFC - \(AP-PAY\)-)Tj 1 0 0 1 199.9 324.4 Tm
        // (HOBBS OF HURST         HASSOCKS)Tj 1 0 0 1 149.5 316.3 Tm
        // (GBR)Tj 1 0 0 1 316.5 324.4 Tm
        // (GBP)Tj 1 0 0 1 427.9 324.4 Tm
        // (30.03)Tj 1 0 0 1 504.7 324.4 Tm
        // (139.50)Tj 1 0 0 1 523.2 324.4 Tm
        // (CR)Tj 1 0 0 1 523.9 587.2 Tm
        // Line 1: Transaction processed date
        // Line 2: Transaction date
        // Line 3: Payment method (optional: only present if Apple Pay)
        // Line 4: Merchant Description
        // Line 5: Transaction country (optional: sometimes this is included in merchant description)
        // Line 6: Transaction currency (optional: sometimes this is included in merchant description)
        // Line 7: Transaction amount (optional: foreign currency)
        // Line 8: Transaction amount (local currency)
        // Line 9: Credit or debit (optional: only present if it's a credit, otherwise assume debit)

        // Parse out the date, description and amount
        let (input, _) = Parser::take_transaction_date(input)?;
        let (input, (transaction_date, transaction_month)) = Parser::take_transaction_date(input)?;
        let (input, _) = opt(Parser::take_transaction_method)(input)?;
        let (input, transaction_desc) = Parser::take_transaction_desc(input)?;
        let (input, _) = opt(Parser::take_transaction_location)(input)?;
        let (input, _) = opt(Parser::take_transaction_location)(input)?;
        let (input, transaction_amnt_foreign) = Parser::take_transaction_amnt(input)?;
        let (input, transaction_amnt_local) = opt(Parser::take_transaction_amnt)(input)?;
        let (input, credit) = opt(Parser::take_is_credit)(input)?;

        // If this is a foreign transaction, then the foreign currency is
        // listed first, then the local currency.
        // If this is a local transaction, then just the local currency is listed.
        let mut transaction_amnt = transaction_amnt_foreign;
        if let Some(ta) = transaction_amnt_local {
            transaction_amnt = ta;
        }

        // Clean the merchant up (remove location etc)
        let details = Parser::clean_merchant_details(transaction_desc);

        // Format the transaction date
        let day = String::from_utf8_lossy(transaction_date);
        let month = String::from_utf8_lossy(transaction_month);
        let now: DateTime<Utc> = Utc::now();
        let datestr = format!("{} {} {}", day, month, now.year());
        let date = NaiveDate::parse_from_str(&datestr, "%d %b %Y")
            .unwrap()
            .and_hms(0, 0, 0);

        let mut transaction = Transaction {
            id: None,
            amount: transaction_amnt,
            date: date.timestamp(),
            details,
            category: None,
        };
        transaction.id = Some(transaction.hash());

        Ok((input, (credit.is_some(), transaction)))
    }

    fn format_amount(amount: (&[u8], &[u8])) -> Decimal {
        Decimal::from_str(&format!(
            "{}.{}",
            String::from_utf8_lossy(amount.0).replace(",", ""),
            String::from_utf8_lossy(amount.1)
        ))
        .unwrap()
    }

    fn take_transaction_date(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
        let (input, _) = take_until("(")(input)?;
        let (input, _) = tag("(")(input)?;
        let (input, day) = take_while1(is_digit)(input)?;
        let (input, month) = alt((
            tag("JAN"),
            tag("FEB"),
            tag("MAR"),
            tag("APR"),
            tag("MAY"),
            tag("JUN"),
            tag("JUL"),
            tag("AUG"),
            tag("SEP"),
            tag("OCT"),
            tag("NOV"),
            tag("DEC"),
        ))(input)?;
        let (input, _) = tag(")")(input)?;
        Ok((input, (day, month)))
    }

    fn take_transaction_method(input: &[u8]) -> IResult<&[u8], &[u8]> {
        let (input, _) = take_until("(")(input)?;
        alt((tag("(IAP - \\(AP-PAY\\)-)"), tag("(NFC - \\(AP-PAY\\)-)")))(input)
    }

    fn take_transaction_location(input: &[u8]) -> IResult<&[u8], &[u8]> {
        let (input, _) = take_until("(")(input)?;
        let (input, _) = tag("(")(input)?;
        let (input, location) = take_while(Parser::is_alphanumeric_or_whitespace)(input)?;
        let (input, _) = tag(")")(input)?;
        Ok((input, location))
    }

    fn is_alphanumeric_or_whitespace(input: u8) -> bool {
        is_alphanumeric(input) || is_space(input)
    }

    fn take_transaction_desc(input: &[u8]) -> IResult<&[u8], &[u8]> {
        let (input, _) = take_until("(")(input)?;
        let (input, _) = tag("(")(input)?;
        let (input, desc) = take_until(")")(input)?;
        let (input, _) = tag(")")(input)?;
        Ok((input, desc))
    }

    fn take_transaction_amnt(input: &[u8]) -> IResult<&[u8], Decimal> {
        let (input, _) = take_until("(")(input)?;
        let (input, _) = tag("(")(input)?;
        let (input, dirhams) = take_while1(Parser::is_digit_or_comma)(input)?;
        let (input, _) = tag(".")(input)?;
        let (input, fils) = take_while1(is_digit)(input)?;
        let (input, _) = tag(")")(input)?;
        Ok((input, Parser::format_amount((dirhams, fils))))
    }

    fn take_is_credit(input: &[u8]) -> IResult<&[u8], &[u8]> {
        let (input, _) = take_until("(")(input)?;
        tag("(CR)")(input)
    }

    fn is_digit_or_comma(chr: u8) -> bool {
        is_digit(chr) || chr == b','
    }

    fn parse_streams(input: &[u8]) -> IResult<&[u8], Vec<Stream>> {
        many0(Parser::parse_stream)(input)
    }

    fn parse_stream<'a>(input: &'a [u8]) -> IResult<&'a [u8], Stream> {
        // Parse out the stream length
        let (input, length) = Parser::parse_stream_length(input)?;

        // Parse out the stream's binary data
        let (input, (_, _, bytes)) =
            tuple((take_until("stream\n"), take(7usize), take(length)))(input)?;

        Ok((
            input,
            Stream {
                bytes: bytes.to_vec(),
            },
        ))
    }

    fn parse_stream_length<'a>(input: &'a [u8]) -> IResult<&'a [u8], usize> {
        match tuple((take_until("Length "), take(7usize), take_until("\n")))(input) {
            Ok((input, (_, _, length_bytes))) => {
                let length = String::from_utf8_lossy(length_bytes);
                Ok((input, length.parse().unwrap()))
            }
            Err(e) => Err(e),
        }
    }

    fn clean_merchant_details(input: &[u8]) -> String {
        let merchant = String::from_utf8_lossy(input).to_string();
        String::from(merchant.split("  ").collect::<Vec<&str>>()[0])
    }
}
