mod clients;
mod header;
mod scheme;

pub type Compressor<W> = clients::Compressor<W>;
pub type Decompressor<W> = clients::Decompressor<W>;

/// Given a buffer of bytes, returns a Vec of slices of each compressed frame in the buffer.
pub fn split_frames(data: &[u8]) -> Vec<&[u8]> {
    clients::split_frames(data)
}
