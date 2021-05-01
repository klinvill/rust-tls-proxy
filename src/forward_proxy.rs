use crate::compression::{split_frames, Compressor, Decompressor, Direction};
use crate::errors::*;
use crate::iostream::IoStream;
use crate::reverse_proxy;
use dns_lookup::lookup_addr;
use error_chain::bail;
use nix::sys::socket;
use std::fs::File;
use std::io::{BufReader, Write};
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
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

async fn proxy_conn(
    mut read_conn: ReadHalf<IoStream>,
    mut write_conn: WriteHalf<IoStream>,
    compress_direction: Option<Direction>,
) {
    let mut buf = vec![0; 1024];

    loop {
        // proxy from the read connection to the write connection
        match read_conn.read(&mut buf).await {
            Ok(0) => {
                println!("Read connection closed");
                break;
            }
            Ok(n) => {
                // TODO: cleanup match statements
                let comp_buf = match compress_direction {
                    Some(Direction::Decompress) => {
                        // TODO: add graceful error handling
                        let decomp_frames = split_frames(&buf[..n]);
                        decomp_frames
                            .iter()
                            .flat_map(|frame| {
                                let mut decomp = Decompressor::new(Vec::new());
                                decomp.write_all(frame).expect("Decompression error");
                                decomp.finish().expect("Decompression error")
                            })
                            .collect()
                    }
                    Some(Direction::Compress) => {
                        let compressed_buf = Vec::new();
                        let mut comp = Compressor::new(compressed_buf);
                        match comp.write_all(&buf[..n]) {
                            Err(e) => {
                                eprintln!("Compression error: {}", e.to_string());
                                break;
                            }
                            _ => (),
                        };
                        match comp.finish() {
                            Ok(comp_buf) => comp_buf,
                            Err(e) => {
                                eprintln!("Compression error: {}", e.to_string());
                                break;
                            }
                        }
                    }
                    None => vec![],
                };

                let write_buffer = match compress_direction {
                    Some(_) => &comp_buf,
                    None => &buf[..n],
                };

                if let Err(_) = write_conn.write_all(write_buffer).await {
                    eprintln!("Error sending to write connection");
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

#[cfg(test)]
mod tests {
    use crate::compression::{split_frames, Compressor, Decompressor, Direction};
    use crate::forward_proxy::{proxy_conn, IoStream};
    use std::io::Write;
    use tokio;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    struct TestProxy {
        reader: TcpStream,
        writer: TcpStream,
    }

    /// Helper function to create proxied tcp connections. Returns a tuple of the connections to
    /// write to the proxy and read from the proxy respectively
    async fn setup_proxy(compress_direction: Option<Direction>) -> TestProxy {
        let in_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        let out_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        let in_send_conn = TcpStream::connect(in_listener.local_addr().unwrap())
            .await
            .unwrap();
        let (in_recv_conn, _) = in_listener.accept().await.unwrap();

        let out_send_conn = TcpStream::connect(out_listener.local_addr().unwrap())
            .await
            .unwrap();
        let (out_recv_conn, _) = out_listener.accept().await.unwrap();

        let (in_recv_read, _) = split::<IoStream>(IoStream::from(in_recv_conn));
        let (_, out_send_write) = split::<IoStream>(IoStream::from(out_send_conn));

        tokio::spawn(async move {
            proxy_conn(in_recv_read, out_send_write, compress_direction).await;
        });

        TestProxy {
            reader: in_send_conn,
            writer: out_recv_conn,
        }
    }

    #[tokio::test]
    async fn proxy_content() {
        let message = "Hello world! This is message should be proxied.".as_bytes();
        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(None).await;

        test_proxy.reader.write_all(&message).await.unwrap();
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, message);
    }

    #[tokio::test]
    async fn proxy_compressed_content() {
        let message = "Hello world! This is message should be proxied and compressed.".as_bytes();
        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(Some(Direction::Compress)).await;

        test_proxy.reader.write_all(&message).await.unwrap();
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        let expected_message = Vec::new();
        let mut ref_compressor = Compressor::new(expected_message);
        ref_compressor.write_all(&message).unwrap();

        assert_eq!(received, ref_compressor.finish().unwrap());
    }

    #[tokio::test]
    async fn proxy_large_compressed_content() {
        // ~2kB message
        let message = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam risus metus, vulputate sed erat non, maximus accumsan augue. Ut eu aliquet urna, sed mollis lectus. Vivamus eu egestas lectus. Donec commodo diam vehicula nisl iaculis, at scelerisque est efficitur. Pellentesque sed dolor arcu. Nullam semper quam risus, quis lobortis sapien mollis vitae. Fusce egestas ante nisl, ac bibendum mi faucibus ac. Phasellus eu libero orci. Cras dignissim in nibh quis eleifend. Duis mattis fermentum nulla ac aliquet. Cras et orci quis erat fermentum auctor et in mauris. Ut ornare, elit a blandit imperdiet, nibh sapien dapibus sapien, non faucibus diam arcu fermentum nunc. Proin feugiat pharetra lectus vitae semper. Fusce sit amet tortor mattis, hendrerit ex nec, iaculis risus.

Nam est nibh, semper sit amet gravida eu, efficitur in tortor. Aenean vel leo vitae enim scelerisque porta at et nibh. Nulla malesuada vel ipsum placerat varius. Aliquam facilisis, dolor quis ultrices condimentum, nisl metus consequat purus, non vulputate odio odio at justo. Fusce rhoncus neque arcu, et venenatis lacus vestibulum at. Nullam tristique tincidunt nunc. Ut mollis sem non turpis accumsan, et volutpat quam suscipit. Cras metus libero, commodo vitae purus vulputate, scelerisque molestie mi. Etiam posuere orci id turpis suscipit egestas. Nunc id faucibus risus.

Duis quis neque sit amet turpis ullamcorper pretium a et turpis. In ultrices eros sit amet odio venenatis varius. Vestibulum id sem iaculis dolor ornare egestas eu sit amet nunc. Integer elit lorem, pretium vestibulum euismod in, imperdiet porttitor nisl. In accumsan elit non rutrum euismod. Integer turpis sem, lobortis non laoreet id, mattis at metus. Sed hendrerit volutpat dui ut consectetur.

Duis efficitur, lacus a condimentum rhoncus, justo ex tristique neque, fermentum imperdiet tortor ex a ante. Mauris a tortor nec sapien volutpat porttitor. Praesent purus erat, viverra sed rhoncus eget, sodales ac felis. Integer scelerisque leo gravida.".as_bytes();
        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(Some(Direction::Compress)).await;

        test_proxy.reader.write_all(&message).await.unwrap();
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        let compression_frames = split_frames(&received);
        // The forward proxy should receive the data as 2 different messages, each will be
        // compressed separately.
        // TODO: should make this more maintainable by not hardcoding it to expect 2 chunks but
        //  rather the number of chunks that should be produced
        assert_eq!(compression_frames.len(), 2);

        let decompressed_data: Vec<u8> = compression_frames
            .iter()
            .flat_map(|frame| {
                let mut ref_decompressor = Decompressor::new(Vec::new());
                ref_decompressor.write_all(frame).unwrap();
                ref_decompressor.finish().unwrap()
            })
            .collect();

        assert_eq!(message, decompressed_data);
    }
}
