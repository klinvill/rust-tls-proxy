use std::net::{SocketAddr, TcpListener};
use crate::errors::*;

pub const DEFAULT_PORT : u16 = 8080;

pub fn run(
    _local_addr: SocketAddr, 
    _compress: bool, 
    _encrypt: bool
) -> Result<()>
{
    Ok(())
}
