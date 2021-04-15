use std::io::prelude::*;
use flate2::write::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;
use crate::compression::header::Header;
use crate::compression::scheme::Scheme;

pub struct Compressor<C: Write> {
    encoder: C,
    wrote_header: bool,
}

// TODO: can assume deflate for now, add other compression schemes
impl<W: Write> Compressor<DeflateEncoder<W>> {
    pub fn new(writer: W) -> Compressor<DeflateEncoder<W>> {
        Compressor {
            encoder: DeflateEncoder::new(writer, Compression::default()),
            wrote_header: false,
        }
    }

    pub fn compress(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if !self.wrote_header {
            let header = Header::new(Scheme::DEFLATE);
            let header_bytes = match header.to_bytes() {
                Some(data) => data,
                None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not convert header to bytes")),
            };
            self.encoder.get_mut().write_all(&header_bytes)?;
            self.wrote_header = true;
        }

        self.encoder.write_all(buf)
    }

    pub fn finish(self) -> std::io::Result<W> {
        self.encoder.finish()
    }
}

impl<W: Write> Compressor<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.encoder.flush()
    }
}



#[cfg(test)]
mod tests {
    // use crate::compression::compressor::{Compressor, compress};
    use crate::compression::compressor::{Compressor};
    use flate2::write::DeflateEncoder;
    use std::io::prelude::*;
    use crate::compression::header::Header;
    use crate::compression::scheme::Scheme;

    # [test]
    fn compress_once() {
        let mut compressor = Compressor::new(Vec::new());
        let message = "Hello world! This is quite compressed....".as_bytes();

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        reference_enc.write_all(message).unwrap();
        let expected_header = Header::new(Scheme::DEFLATE);
        let expected_compressed = reference_enc.finish().unwrap();
        let expected_result = [expected_header.to_bytes().unwrap(), expected_compressed].concat();

        compressor.compress(message);
        let result = compressor.finish().unwrap();

        assert!(result.len() > 0);
        assert_eq!(result, expected_result);
    }

    # [test]
    fn compress_multiple_payloads() {
        let mut compressor = Compressor::new(Vec::new());
        let messages = ["Hello world!".as_bytes(), " This is quite compressed....".as_bytes()];

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        for message in messages.iter() {
            reference_enc.write_all(message).unwrap();
        }
        let expected_header = Header::new(Scheme::DEFLATE);
        let expected_compressed = reference_enc.finish().unwrap();
        let expected_result = [expected_header.to_bytes().unwrap(), expected_compressed].concat();

        for message in messages.iter() {
            compressor.compress(message);
        }
        let result = compressor.finish().unwrap();

        assert!(result.len() > 0);
        assert_eq!(result, expected_result);
    }
}
