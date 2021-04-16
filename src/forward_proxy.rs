use std::error::Error;
use std::net::TcpListener;

pub const DEFAULT_PORT : u16 = 8080;

pub fn run(listen_socket: TcpListener, compress: bool, encrypt: bool) -> Result<(), Box<Error>>
{
    Ok(())
}
