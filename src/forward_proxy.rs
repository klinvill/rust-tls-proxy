use crate::compression::Direction;
use crate::errors::*;
use crate::iostream::IoStream;
use crate::proxy_common::proxy_conn;
use crate::reverse_proxy;
use dns_lookup::lookup_addr;
use error_chain::bail;
use nix::sys::socket;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Arc;
use tokio::io::split;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{rustls::ClientConfig, webpki::DNSNameRef, TlsConnector, TlsStream};

pub const PROXY_REDIR_PORT: u16 = 8080;

pub fn run(
    local_addr: SocketAddr,
    compress: bool,
    encrypt: bool,
    root_certs_path: Option<&Path>,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().chain_err(|| "failed to create tokio runtime")?;
    rt.block_on(run_async(local_addr, compress, encrypt, root_certs_path))
}

pub async fn run_async(
    local_addr: SocketAddr,
    compress: bool,
    encrypt: bool,
    root_certs_path: Option<&Path>,
) -> Result<()> {
    println!("opening listener socket on {}", local_addr);
    let listen_socket = TcpListener::bind(local_addr)
        .await
        .chain_err(|| format!("error opening listener socket on {}", local_addr))?;

    socket::setsockopt(
        listen_socket.as_raw_fd(),
        socket::sockopt::IpTransparent,
        &true,
    )?;

    forward_proxy(listen_socket, compress, encrypt, root_certs_path).await
}

/// Note: this function allows for a custom TcpListener to be provided. Most users will either want
/// to call run() or run_async() which sets the IP_TRANSPARENT option for the socket. This function
/// is primarily useful for testing without the IP_TRANSPARENT option.
pub async fn forward_proxy(
    listen_socket: TcpListener,
    compress: bool,
    encrypt: bool,
    root_certs_path: Option<&Path>,
) -> Result<()> {
    let mut tls_config = ClientConfig::new();

    if encrypt {
        let mut root_file = BufReader::new(File::open(
            root_certs_path.ok_or("Must provide a root cert path if encrypt is set.")?,
        )?);
        tls_config
            .root_store
            .add_pem_file(&mut root_file)
            .map_err(|_| "Couldn't parse root cert file.")?;
        if tls_config.root_store.is_empty() {
            bail!("No root certs added to store.")
        }
    }

    let tls_config_ref = Arc::new(tls_config);
    loop {
        let (from_conn, from_addr) = listen_socket
            .accept()
            .await
            .chain_err(|| format!("error accepting connection"))?;
        println!("connection received from {}", from_addr);

        match socket::getsockname(from_conn.as_raw_fd()) {
            Ok(socket::SockAddr::Inet(inet_addr)) => {
                println!("connection destined to {}", inet_addr);

                let to_addr = SocketAddr::new(inet_addr.ip().to_std(), reverse_proxy::HTTPS_PORT);

                let to_tcp_conn = TcpStream::connect(to_addr).await?;
                let to_conn = match encrypt {
                    false => IoStream::from(to_tcp_conn),
                    true => {
                        let string_dnsname = lookup_addr(&inet_addr.ip().to_std())?;
                        let dnsname = DNSNameRef::try_from_ascii_str(&string_dnsname)?;
                        let connector = TlsConnector::from(Arc::clone(&tls_config_ref));
                        IoStream::from(TlsStream::from(
                            connector.connect(dnsname, to_tcp_conn).await?,
                        ))
                    }
                };

                println!("connection opened to {}", to_addr);
                let (client_read, client_write) = split::<IoStream>(IoStream::from(from_conn));
                let (server_read, server_write) = split::<IoStream>(to_conn);

                tokio::spawn(async move {
                    proxy_conn(
                        client_read,
                        server_write,
                        if compress {
                            Some(Direction::Compress)
                        } else {
                            None
                        },
                    )
                    .await;
                });
                tokio::spawn(async move {
                    proxy_conn(
                        server_read,
                        client_write,
                        if compress {
                            Some(Direction::Decompress)
                        } else {
                            None
                        },
                    )
                    .await;
                });
            }
            _ => eprintln!("Failed to get destination address"),
        }
    }
}
