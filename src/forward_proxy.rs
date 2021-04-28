use crate::compression::{Compressor, Decompressor};
use crate::errors::*;
use nix::sys::socket;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use std::thread;

use crate::reverse_proxy;

pub const PROXY_REDIR_PORT: u16 = 8080;

pub fn run(local_addr: SocketAddr, compress: bool, encrypt: bool) -> Result<()> {
    println!("opening listener socket on {}", local_addr);
    let listen_socket = TcpListener::bind(local_addr)
        .chain_err(|| format!("error opening listener socket on {}", local_addr))?;

    socket::setsockopt(
        listen_socket.as_raw_fd(),
        socket::sockopt::IpTransparent,
        &true,
    )?;

    loop {
        let (from_conn, from_addr) = listen_socket
            .accept()
            .chain_err(|| format!("error accepting connection"))?;
        println!("connection received from {}", from_addr);

        match socket::getsockname(from_conn.as_raw_fd()) {
            Ok(socket::SockAddr::Inet(inet_addr)) => {
                println!("connection destined to {}", inet_addr);

                let to_addr = SocketAddr::new(inet_addr.ip().to_std(), reverse_proxy::HTTPS_PORT);

                if let Ok(to_conn) = TcpStream::connect(to_addr) {
                    println!("connection opened to {}", to_addr);
                    let client_write = from_conn.try_clone().unwrap();
                    let client_read = from_conn;

                    let server_write = to_conn.try_clone().unwrap();
                    let server_read = to_conn;

                    thread::spawn(move || {
                        to_server(
                            client_read,
                            server_write.try_clone().unwrap(),
                            compress,
                            encrypt,
                        );
                        let _ = server_write.shutdown(Shutdown::Both);
                    });
                    thread::spawn(move || {
                        to_client(
                            server_read,
                            client_write.try_clone().unwrap(),
                            compress,
                            encrypt,
                        );
                        let _ = client_write.shutdown(Shutdown::Both);
                    });
                } else {
                    eprintln!("failed to connect to {}", to_addr);
                }
            }
            _ => eprintln!("Failed to get destination address"),
        }
    }
}

fn to_server(mut read_conn: impl Read, write_conn: impl Write, compress: bool, _encrypt: bool) {
    let mut buf = vec![0; 1024];
    let mut writer: Box<dyn Write> = match compress {
        true => Box::new(Compressor::new(write_conn)),
        false => Box::new(write_conn),
    };

    loop {
        // echo client to server
        match read_conn.read(&mut buf) {
            Ok(0) => {
                println!("Client closed connection");
                break;
            }
            Ok(n) => {
                println!("From client: {:?}", &buf[..n]);
                let _ = writer.write_all(&buf[..n]);
                if let Err(_) = writer.flush() {
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
}

fn to_client(mut read_conn: impl Read, write_conn: impl Write, compress: bool, _encrypt: bool) {
    let mut buf = vec![0; 1024];
    let mut writer: Box<dyn Write> = match compress {
        true => Box::new(Decompressor::new(write_conn)),
        false => Box::new(write_conn),
    };

    loop {
        // echo server to client
        match read_conn.read(&mut buf) {
            Ok(0) => {
                println!("Server closed connection");
                break;
            }
            Ok(n) => {
                println!("From server: {:?}", &buf[..n]);
                let _ = writer.write_all(&buf[..n]);
                if let Err(_) = writer.flush() {
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
}
