use crate::pdf::PDFStream;
use crate::{Statement, Transaction};

use nom::bytes::complete::*;
use nom::multi::many0;
use nom::sequence::*;
use nom::IResult;

pub fn parse_statement(input: &[u8]) -> IResult<&[u8], Statement> {
	Ok((input, Statement::default()))
}

pub(crate) fn parse_transactions(input: &[u8]) -> IResult<&[u8], Vec<Transaction>> {
	many0(parse_transaction)(input)
}

pub(crate) fn parse_transaction(input: &[u8]) -> IResult<&[u8], Transaction> {
	// [(  )] TJ 1 0 0 1 60.2 538.3 Tm
	// (07AUG)Tj 1 0 0 1 110.6 538.3 Tm
	// (06AUG)Tj 1 0 0 1 150.2 538.3 Tm
	// (SUN AND SAND SPORTS ST DUBAI         AE)Tj 1 0 0 1 505.4 538.3 Tm
	// (399.00)Tj 1 0 0 1 523.9 538.3 Tm

	Ok((input, Transaction::default()))
}

pub(crate) fn parse_streams(input: &[u8]) -> IResult<&[u8], Vec<PDFStream>> {
	many0(parse_stream)(input)
}

pub(crate) fn parse_stream(input: &[u8]) -> IResult<&[u8], PDFStream> {
	// Parse out the stream length
	let (remaining, length) = parse_stream_length(input)?;

	// Parse out the stream's binary data
	let matches = tuple((take_until("stream\n"), take(7usize), take(length)))(remaining);

	match matches {
		Ok((remaining, (_, _, stream))) => Ok((remaining, PDFStream { bytes: stream })),
		Err(e) => Err(e),
	}
}

pub(crate) fn parse_stream_length(input: &[u8]) -> IResult<&[u8], usize> {
	match tuple((take_until("Length "), take(7usize), take_until("\n")))(input) {
		Ok((remaining, (_, _, length_bytes))) => {
			let length = String::from_utf8_lossy(length_bytes);
			Ok((remaining, length.parse().unwrap()))
		}
		Err(e) => Err(e),
	}
}
