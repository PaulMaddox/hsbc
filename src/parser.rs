use crate::{Statement, Transaction};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use flate2::read::DeflateDecoder;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;
use std::io::prelude::*;

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
      // Pass out the textboxes from the PDF
      let (_, textboxes) = Parser::parse_textboxes(&stream.bytes).unwrap();
      for textbox in textboxes.iter() {
        // Does this textbox contain a statement overview?

        // Does this textbox contain a transaction?
        if let Ok((_, transaction)) = Parser::parse_transaction(textbox) {
          statement.transactions.push(transaction);
        }
      }
    }

    Ok(statement)
  }

  fn parse_textboxes(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    many0(Parser::parse_textbox)(input)
  }

  fn parse_textbox(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, (textbox, _)) = tuple((take_until("TJ"), take(2usize)))(input)?;
    Ok((input, textbox))
  }

  fn parse_transaction(input: &[u8]) -> IResult<&[u8], Transaction> {
    // Parse out the date, description and amount
    let (input, _) = Parser::take_transaction_date(input)?;
    let (input, (transaction_date, transaction_month)) = Parser::take_transaction_date(input)?;
    let (input, transaction_desc) = Parser::take_transaction_desc(input)?;
    let (input, (transaction_dirhams, transaction_fils)) = Parser::take_transaction_amnt(input)?;
    let (input, credit) = nom::combinator::opt(Parser::take_is_credit)(input)?;

    // Format the transaction date
    let day = String::from_utf8_lossy(transaction_date);
    let month = String::from_utf8_lossy(transaction_month);
    let now: DateTime<Utc> = Utc::now();
    let datestr = format!("{} {} {}", day, month, now.year());
    let date = NaiveDate::parse_from_str(&datestr, "%d %b %Y")
      .unwrap()
      .and_hms(0, 0, 0);

    // Format the transaction amount
    let mut transaction_amnt: f64 = format!(
      "{}.{}",
      String::from_utf8_lossy(transaction_dirhams).replace(",", ""),
      String::from_utf8_lossy(transaction_fils)
    )
    .parse()
    .unwrap();

    if credit.is_none() {
      // This is not a credit (marked with CR in the statement)
      // so it must be a debit. Make the transaction amount negative
      transaction_amnt = 0f64 - transaction_amnt;
    }

    Ok((
      input,
      Transaction {
        amount: transaction_amnt,
        date: date.timestamp(),
        details: String::from_utf8_lossy(transaction_desc).to_string(),
      },
    ))
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

  fn take_transaction_desc(input: &[u8]) -> IResult<&[u8], &[u8]> {
    // println!(
    //   "Looking for transaction in: {:#?}",
    //   String::from_utf8_lossy(input)
    // );

    let (input, _) = take_until("(")(input)?;
    let (input, _) = tag("(")(input)?;
    // let (input, desc) = take_while1(Parser::is_alphanumeric_or_whitespace)(input)?;
    let (input, desc) = take_until(")")(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, desc))
  }

  fn take_transaction_amnt(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    let (input, _) = take_until("(")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, dirhams) = take_while1(Parser::is_digit_or_comma)(input)?;
    let (input, _) = tag(".")(input)?;
    let (input, fils) = take_while1(is_digit)(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, (dirhams, fils)))
  }

  fn take_is_credit(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, _) = take_until("(")(input)?;
    tag("(CR)")(input)
  }

  fn is_digit_or_comma(chr: u8) -> bool {
    is_digit(chr) || chr == b','
  }

  fn is_alphanumeric_or_whitespace(chr: u8) -> bool {
    is_alphanumeric(chr) || is_space(chr)
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
}
