use std::net::{TcpListener, SocketAddr};
use crate::errors::*;

pub const DEFAULT_PORT : u16 = 443;

pub fn run(
    _listen_socket: TcpListener, 
    _server_ips: Vec<SocketAddr>, 
    _compress: bool, 
    _encrypt: bool,
) -> Result<()>
{
    Ok(())
}
