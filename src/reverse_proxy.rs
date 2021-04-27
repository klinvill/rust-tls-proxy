use crate::errors::*;
use std::net::SocketAddr;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};

pub const HTTPS_PORT: u16 = 9443;

pub fn run(
    local_addr: SocketAddr,
    server_ips: Vec<SocketAddr>,
    compress: bool,
    encrypt: bool,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().chain_err(|| "failed to create tokio runtime")?;

    rt.block_on(async {
        let mut server_carousel = server_ips.iter().cycle();

        println!("opening listener socket on {}", local_addr);

        let listen_socket = TcpListener::bind(local_addr)
            .await
            .chain_err(|| format!("error opening listener socket on {}", local_addr))?;

        loop {
            let (from_conn, from_addr) = listen_socket
                .accept()
                .await
                .chain_err(|| format!("error accepting connection"))?;
            println!("connection received from {}", from_addr);

            let to_addr = server_carousel
                .next()
                .chain_err(|| "server carousel failed to provide server addr")?
                .clone();

            if let Ok(to_conn) = TcpStream::connect(to_addr).await {
                println!("connection opened to {}", to_addr);

                let (client_read, client_write) = split::<TcpStream>(from_conn);
                let (server_read, server_write) = split::<TcpStream>(to_conn);

                tokio::spawn(async move {
                    to_server(client_read, server_write, compress, encrypt).await;
                });
                tokio::spawn(async move {
                    to_client(server_read, client_write, compress, encrypt).await;
                });
            } else {
                eprintln!("failed to connect to {}", to_addr);
            }
        }
    })
}

async fn to_server(
    mut read_conn: ReadHalf<TcpStream>,
    mut write_conn: WriteHalf<TcpStream>,
    _compress: bool,
    _encrypt: bool,
) {
    let mut buf = vec![0; 1024];

    loop {
        // echo client to server
        match read_conn.read(&mut buf).await {
            Ok(0) => {
                println!("Client closed connection");
                break;
            }
            Ok(n) => {
                if let Err(_) = write_conn.write_all(&buf[..n]).await {
                    eprintln!("Error sending to server");
                    break;
                }
            }
            Err(_) => {
                eprintln!("Socket error");
                break;
            }
        }
    }

    let _ = write_conn.shutdown().await;
}

async fn to_client(
    mut read_conn: ReadHalf<TcpStream>,
    mut write_conn: WriteHalf<TcpStream>,
    _compress: bool,
    _encrypt: bool,
) {
    let mut buf = vec![0; 1024];

    loop {
        // echo server to client
        match read_conn.read(&mut buf).await {
            Ok(0) => {
                println!("Server closed connection");
                break;
            }
            Ok(n) => {
                if let Err(_) = write_conn.write_all(&buf[..n]).await {
                    eprintln!("Error sending to client");
                    break;
                }
            }
            Err(_) => {
                eprintln!("Socket error");
                break;
            }
        }
    }

    let _ = write_conn.shutdown().await;
}
