use flate2::read::DeflateDecoder;
use std::io::prelude::*;

#[derive(Clone, Default, Debug)]
pub struct PDFStream<'a> {
    pub bytes: &'a [u8],
}

impl<'a> PDFStream<'a> {
    pub fn decompress(&self, output: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        let mut compressed = self.bytes.to_vec();

        // Remove the first two bytes of the stream
        compressed.drain(0..2);

        let mut decoder = DeflateDecoder::new(compressed.as_slice());
        decoder.read_to_end(output)
    }
}
