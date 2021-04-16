use std::net::TcpListener;
use crate::errors::*;

pub const DEFAULT_PORT : u16 = 8080;

pub fn run(
    _listen_socket: TcpListener, 
    _compress: bool, 
    _encrypt: bool
) -> Result<()>
{
    Ok(())
}
