use std::io::Write;

mod clients;
mod header;
mod scheme;

pub type Compressor<W> = clients::Compressor<W>;
pub type Decompressor<W> = clients::Decompressor<W>;

/// Given a buffer of bytes, returns a Vec of slices of each compressed frame in the buffer.
pub fn split_frames(data: &[u8]) -> Vec<&[u8]> {
    clients::split_frames(data)
}

pub enum Direction {
    Compress,
    Decompress,
}

/// Compresses a byte buffer and returns the resulting vec
pub fn compress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut comp = Compressor::new(Vec::new());
    comp.write_all(data)?;
    comp.finish()
}

/// Decompresses a byte buffer and returns the resulting vec
pub fn decompress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decomp = Decompressor::new(Vec::new());
    decomp.write_all(data)?;
    decomp.finish()
}
