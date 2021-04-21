use crate::compression::header::Header;
use crate::compression::scheme::Scheme;
use flate2::write::{DeflateDecoder, DeflateEncoder};
use flate2::Compression;
use std::io::prelude::*;

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

    fn write_header(&mut self) -> std::io::Result<usize> {
        if self.wrote_header {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Header already written",
            ));
        }

        let header = Header::new(Scheme::Deflate);
        let header_bytes = match header.to_bytes() {
            Some(data) => data,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Could not convert header to bytes",
                ))
            }
        };
        // We need to write the header to the underlying writer so the header is not compressed
        let result = self.encoder.get_mut().write(&header_bytes);

        if result.is_ok() {
            self.wrote_header = true;
        }

        result
    }

    pub fn finish(self) -> std::io::Result<W> {
        self.encoder.finish()
    }

    pub fn get_ref(&self) -> &W {
        self.encoder.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.encoder.get_mut()
    }
}

impl<W: Write> Write for Compressor<DeflateEncoder<W>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // We only return the number of bytes written from the input buffer. This is how flate2's
        // write() works: https://github.com/rust-lang/flate2-rs/blob/7546110602fcc934ae506ed8d5cd9516e945d1ee/src/zio.rs#L218
        if !self.wrote_header {
            self.write_header()?;
        }

        self.encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.encoder.flush()
    }
}

pub struct Decompressor<D: Write> {
    decoder: D,
    // TODO: should probably move the header check to the constructor, so that we can construct the
    //  correct kind of decoder
    parsed_header: bool,
}

// TODO: can assume deflate for now, add other compression schemes
impl<W: Write> Decompressor<DeflateDecoder<W>> {
    pub fn new(writer: W) -> Decompressor<DeflateDecoder<W>> {
        Decompressor {
            decoder: DeflateDecoder::new(writer),
            parsed_header: false,
        }
    }

    pub fn finish(self) -> std::io::Result<W> {
        self.decoder.finish()
    }

    pub fn get_ref(&self) -> &W {
        self.decoder.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.decoder.get_mut()
    }

    fn parse_header(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match Header::from_bytes(buf) {
            Some(header) => {
                if header.scheme != Scheme::Deflate {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Only the deflate compression scheme is currently supported",
                    ));
                }
                self.parsed_header = true;
                Ok(Header::serialized_size())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "A compression header must be present in the first few bytes of the buffer",
            )),
        }
    }
}

impl<W: Write> Write for Decompressor<DeflateDecoder<W>> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        // We return the number of bytes parsed from the input buffer, rather than the number of
        // bytes written to the stream. This is how flate2's write() works:
        // https://github.com/rust-lang/flate2-rs/blob/7546110602fcc934ae506ed8d5cd9516e945d1ee/src/zio.rs#L218
        if !self.parsed_header {
            written += self.parse_header(buf)?;
        }

        written += self.decoder.write(&buf[written..])?;

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.decoder.flush()
    }
}

#[cfg(test)]
mod tests {
    use crate::compression::clients::{Compressor, Decompressor};
    use crate::compression::header::Header;
    use crate::compression::scheme::Scheme;
    use flate2::write::{DeflateDecoder, DeflateEncoder};
    use std::io::prelude::*;
    use std::io::BufWriter;

    #[test]
    fn compress_once() {
        let mut compressor = Compressor::new(Vec::new());
        let message = "Hello world! This is quite compressed....".as_bytes();

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        reference_enc.write_all(message).unwrap();
        let expected_header = Header::new(Scheme::Deflate);
        let expected_compressed = reference_enc.finish().unwrap();
        let expected_result = [expected_header.to_bytes().unwrap(), expected_compressed].concat();

        compressor.write_all(message).unwrap();
        let result = compressor.finish().unwrap();

        assert!(result.len() > 0);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn compress_multiple_payloads() {
        let mut compressor = Compressor::new(Vec::new());
        let messages = [
            "Hello world!".as_bytes(),
            " This is quite compressed....".as_bytes(),
        ];

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        for message in messages.iter() {
            reference_enc.write_all(message).unwrap();
        }
        let expected_header = Header::new(Scheme::Deflate);
        let expected_compressed = reference_enc.finish().unwrap();
        let expected_result = [expected_header.to_bytes().unwrap(), expected_compressed].concat();

        for message in messages.iter() {
            compressor.write_all(message).unwrap();
        }
        let result = compressor.finish().unwrap();

        assert!(result.len() > 0);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn can_wrap_compressor_write() {
        let compressor = Compressor::new(Vec::new());
        let mut writer = BufWriter::new(compressor);
        let message = "Hello world! This is quite compressed....".as_bytes();

        let expected_header = Header::new(Scheme::Deflate);
        let mut reference_dec = DeflateDecoder::new(Vec::new());

        writer.write_all(message).unwrap();
        writer.flush().unwrap();
        let result = writer.get_ref().get_ref();

        let header_only = &result[..Header::serialized_size()];
        let buf_only = &result[Header::serialized_size()..];

        assert_eq!(header_only, expected_header.to_bytes().unwrap());

        reference_dec.write_all(buf_only).unwrap();
        reference_dec.flush().unwrap();
        let decompressed_result = reference_dec.get_ref();

        assert!(result.len() > 0);
        assert_eq!(decompressed_result, message);
    }

    #[test]
    fn decompress_once() {
        let expected_message = "Hello world! This is quite compressed....".as_bytes();

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        reference_enc.write_all(expected_message).unwrap();
        let header = Header::new(Scheme::Deflate);
        let compressed_data = reference_enc.finish().unwrap();
        let compressed_payload = [header.to_bytes().unwrap(), compressed_data].concat();

        let mut decompressor = Decompressor::new(Vec::new());
        decompressor.write_all(&compressed_payload).unwrap();
        let result = decompressor.finish().unwrap();

        assert!(result.len() > 0);
        assert_eq!(result, expected_message);
    }

    #[test]
    fn decompress_split_payloads() {
        let expected_message = "Hello world! This is quite compressed....".as_bytes();

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        reference_enc.write_all(expected_message).unwrap();
        let header = Header::new(Scheme::Deflate);
        let compressed_data = reference_enc.finish().unwrap();
        let full_payload: Vec<u8> = [header.to_bytes().unwrap(), compressed_data].concat();
        let payload_chunks = full_payload.split_at(full_payload.len() / 2);

        let mut decompressor = Decompressor::new(Vec::new());
        for payload in [payload_chunks.0, payload_chunks.1].iter() {
            decompressor.write_all(&payload).unwrap();
        }
        let result = decompressor.finish().unwrap();

        assert!(result.len() > 0);
        assert_eq!(result, expected_message);
    }

    #[test]
    fn decompress_requires_header() {
        let message = "Hello world! This is quite compressed....".as_bytes();

        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        reference_enc.write_all(message).unwrap();
        let compressed_data = reference_enc.finish().unwrap();

        let mut decompressor = Decompressor::new(Vec::new());
        let decompress_result = decompressor.write_all(&compressed_data);
        assert_eq!(
            decompress_result.unwrap_err().kind(),
            std::io::ErrorKind::Other
        );
    }

    #[test]
    fn can_wrap_decompressor_write() {
        let decompressor = Decompressor::new(Vec::new());
        let mut writer = BufWriter::new(decompressor);

        let message = "Hello world! This is quite compressed....".as_bytes();
        let mut reference_enc = DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        reference_enc.write_all(message).unwrap();
        let header = Header::new(Scheme::Deflate);
        let compressed_data = reference_enc.finish().unwrap();
        let compressed_payload = [header.to_bytes().unwrap(), compressed_data].concat();

        writer.write_all(&compressed_payload).unwrap();
        writer.flush().unwrap();
        let result = writer.get_ref().get_ref();

        assert!(result.len() > 0);
        assert_eq!(result, message);
    }
}
