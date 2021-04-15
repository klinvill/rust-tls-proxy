use std::io::prelude::*;

mod compressor;
mod header;
mod scheme;

pub type Compressor<W> = compressor::Compressor<W>;
