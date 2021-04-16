use std::error::Error;
use std::net::{TcpListener, SocketAddr};

pub const DEFAULT_PORT : u16 = 443;

pub fn run(listen_socket: TcpListener, server_ips: Vec<SocketAddr>, compress: bool, encrypt: bool)
    -> Result<(), Box<Error>>
{
    Ok(())
}
