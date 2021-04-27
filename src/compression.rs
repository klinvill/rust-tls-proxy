mod clients;
mod header;
mod scheme;

pub type Compressor<W> = clients::Compressor<W>;
pub type Decompressor<W> = clients::Decompressor<W>;
