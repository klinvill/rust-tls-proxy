use crate::errors::*;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

pub const DEFAULT_PORT : u16 = 9080; 

pub fn run(local_addr: SocketAddr, _compress: bool, _encrypt: bool) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().chain_err(|| "failed to create tokio runtime")?;

    rt.block_on(async {
        println!("opening listener socket on {}", local_addr);
        let listen_socket = TcpListener::bind(local_addr)
            .await
            .chain_err(|| format!("error opening listener socket on {}", local_addr))?;

        loop {
            let (_, from_addr) = listen_socket
                .accept()
                .await
                .chain_err(|| format!("error accepting connection"))?;
            println!("connection received from {}", from_addr);

            // TODO
            // TcpStream::connect
        }
    })
}
