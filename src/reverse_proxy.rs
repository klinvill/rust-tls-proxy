use std::net::{TcpListener, SocketAddrV4};

pub const DEFAULT_PORT : u16 = 443;

pub fn run(listen_socket: TcpListener, server_ips: Vec<SocketAddrV4>, compress: bool, encrypt: bool)
    -> Result<(), String>
{
    Ok(())
}
