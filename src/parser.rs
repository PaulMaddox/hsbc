use crate::{Statement, Transaction};

use flate2::read::DeflateDecoder;
use nom::bytes::complete::*;
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
		for stream in self.decompressed_streams.iter() {
			let (_, transactions) = Parser::parse_transactions(&stream.bytes).unwrap();
			statement.transactions.append(&mut transactions.clone());
		}

		// ... and finally parse out the statement details (starting balance, closing balance etc)

		Ok(statement)
	}

	fn parse_transactions(input: &[u8]) -> IResult<&[u8], Vec<Transaction>> {
		many0(Parser::parse_transaction)(input)
	}

	fn parse_transaction(input: &[u8]) -> IResult<&[u8], Transaction> {
		// [(  )] TJ 1 0 0 1 60.2 538.3 Tm
		// (07AUG)Tj 1 0 0 1 110.6 538.3 Tm
		// (06AUG)Tj 1 0 0 1 150.2 538.3 Tm
		// (SUN AND SAND SPORTS ST DUBAI         AE)Tj 1 0 0 1 505.4 538.3 Tm
		// (399.00)Tj 1 0 0 1 523.9 538.3 Tm

		let (remaining, (_, encoded)) = tuple((take_until("TJ"), take_until("TJ")))(input)?;

		println!("--- TRANSACTION DATA ---");
		println!("{} ", String::from_utf8_lossy(encoded));
		println!("--- END TRANSACTION DATA ---");

		Ok((remaining, Transaction::default()))
	}

	fn parse_streams(input: &[u8]) -> IResult<&[u8], Vec<Stream>> {
		many0(Parser::parse_stream)(input)
	}

	fn parse_stream<'a>(input: &'a [u8]) -> IResult<&'a [u8], Stream> {
		// Parse out the stream length
		let (remaining, length) = Parser::parse_stream_length(input)?;

		// Parse out the stream's binary data
		let (remaining, (_, _, bytes)) =
			tuple((take_until("stream\n"), take(7usize), take(length)))(remaining)?;

		Ok((
			remaining,
			Stream {
				bytes: bytes.to_vec(),
			},
		))
	}

	fn parse_stream_length<'a>(input: &'a [u8]) -> IResult<&'a [u8], usize> {
		match tuple((take_until("Length "), take(7usize), take_until("\n")))(input) {
			Ok((remaining, (_, _, length_bytes))) => {
				let length = String::from_utf8_lossy(length_bytes);
				Ok((remaining, length.parse().unwrap()))
			}
			Err(e) => Err(e),
		}
	}
}
