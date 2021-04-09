use std::io::prelude::*;
use flate2::write::{DeflateDecoder, DeflateEncoder};

// TODO: can assume deflate for now, add other compression schemes
pub struct Client<W: Write> {
    // Lazily initializes decoder and encoder
    decoder: Option<DeflateDecoder<W>>,
    encoder: Option<DeflateEncoder<W>>,
    writer: W,
}

impl<W: Write> Client<W> {
    pub fn new(writer: W) -> Client<W> {
        Client {
            decoder: None,
            encoder: None,
            writer,
        }
    }

    pub fn compress(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // TODO: compress buf and send output to writer (prepended with header record)
        unimplemented!("Compression not implemented yet");
    }

    pub fn decompress(&mut self, buf: &[u8]) -> std::io::Result<usize>{
        // TODO: first check header record for compression presence and type
        // TODO: decompress buf and send output to writer (after removing header record)
        unimplemented!("Decompression not implemented yet");
    }
}
