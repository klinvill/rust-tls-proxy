use std::io::prelude::*;

mod client;
mod header;
mod scheme;

pub type Client<W> = client::Client<W>;
