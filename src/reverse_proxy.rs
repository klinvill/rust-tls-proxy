use crate::compression::Direction;
use crate::errors::*;
use crate::iostream::IoStream;
use crate::proxy_common::proxy_conn;
use error_chain::bail;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::split;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::internal::pemfile;
use tokio_rustls::rustls::{NoClientAuth, ServerConfig};
use tokio_rustls::{TlsAcceptor, TlsStream};

pub const HTTPS_PORT: u16 = 9443;

pub fn run(
    local_addr: SocketAddr,
    server_ips: Vec<SocketAddr>,
    compress: bool,
    encrypt: bool,
    cert_path: Option<&Path>,
    key_path: Option<&Path>,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().chain_err(|| "failed to create tokio runtime")?;

    rt.block_on(run_async(
        local_addr, server_ips, compress, encrypt, cert_path, key_path,
    ))
}

pub async fn run_async(
    local_addr: SocketAddr,
    server_ips: Vec<SocketAddr>,
    compress: bool,
    encrypt: bool,
    cert_path: Option<&Path>,
    key_path: Option<&Path>,
) -> Result<()> {
    let mut server_carousel = server_ips.iter().cycle();

    println!("opening listener socket on {}", local_addr);

    let listen_socket = TcpListener::bind(local_addr)
        .await
        .chain_err(|| format!("error opening listener socket on {}", local_addr))?;

    let mut tls_config = ServerConfig::new(NoClientAuth::new());
    if encrypt {
        let certs = pemfile::certs(&mut BufReader::new(File::open(
            cert_path.ok_or("Must provide a cert path if encrypt is set")?,
        )?))
        .map_err(|_| "Could not load certs")?;
        if certs.is_empty() {
            bail!("Did not read any certs from file")
        }

        let mut keys = pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(
            key_path.ok_or("Must provide a key path if encrypt is set")?,
        )?))
        .map_err(|_| "Could not load key")?;
        let key = match keys.len() {
            1 => keys.remove(0),
            0 => bail!("Did not read any keys from file."),
            _ => bail!("Read multiple keys from file. Only one private key should be provided."),
        };

        tls_config.set_single_cert(certs, key)?;
    }
    let tls_config_acceptor = TlsAcceptor::from(Arc::new(tls_config));

    loop {
        let (from_tcp_conn, from_addr) = listen_socket
            .accept()
            .await
            .chain_err(|| format!("error accepting connection"))?;
        println!("connection received from {}", from_addr);

        let from_conn = match encrypt {
            false => IoStream::from(from_tcp_conn),
            true => IoStream::from(TlsStream::from(
                tls_config_acceptor.clone().accept(from_tcp_conn).await?,
            )),
        };

        let to_addr = server_carousel
            .next()
            .chain_err(|| "server carousel failed to provide server addr")?
            .clone();

        if let Ok(to_conn) = TcpStream::connect(to_addr).await {
            println!("connection opened to {}", to_addr);

            let (client_read, client_write) = split::<IoStream>(from_conn);
            let (server_read, server_write) = split::<IoStream>(IoStream::from(to_conn));

            tokio::spawn(async move {
                proxy_conn(
                    client_read,
                    server_write,
                    if compress {
                        Some(Direction::Decompress)
                    } else {
                        None
                    },
                )
                .await
            });
            tokio::spawn(async move {
                proxy_conn(
                    server_read,
                    client_write,
                    if compress {
                        Some(Direction::Compress)
                    } else {
                        None
                    },
                )
                .await
            });
        } else {
            eprintln!("failed to connect to {}", to_addr);
        }
    }
}
