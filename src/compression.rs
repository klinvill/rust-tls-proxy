use std::io::prelude::*;

mod client;

pub type Client<W> = client::Client<W>;
