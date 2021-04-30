use crate::compression::{split_frames, Decompressor};
use crate::errors::*;
use crate::iostream::IoStream;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{NoClientAuth, ServerConfig};
use tokio_rustls::{TlsAcceptor, TlsStream};

pub const HTTPS_PORT: u16 = 9443;

pub fn run(
    local_addr: SocketAddr,
    server_ips: Vec<SocketAddr>,
    compress: bool,
    encrypt: bool,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().chain_err(|| "failed to create tokio runtime")?;

    rt.block_on(run_async(local_addr, server_ips, compress, encrypt))
}

pub async fn run_async(
    local_addr: SocketAddr,
    server_ips: Vec<SocketAddr>,
    compress: bool,
    encrypt: bool,
) -> Result<()> {
    let mut server_carousel = server_ips.iter().cycle();

    println!("opening listener socket on {}", local_addr);

    let listen_socket = TcpListener::bind(local_addr)
        .await
        .chain_err(|| format!("error opening listener socket on {}", local_addr))?;

    let tls_config = ServerConfig::new(NoClientAuth::new());
    let tls_config_acceptor = TlsAcceptor::from(Arc::new(tls_config));

    loop {
        let (from_tcp_conn, from_addr) = listen_socket
            .accept()
            .await
            .chain_err(|| format!("error accepting connection"))?;
        println!("connection received from {}", from_addr);

        let from_conn = match encrypt {
            false => IoStream::TcpStream(from_tcp_conn),
            true => IoStream::TlsStream(TlsStream::from(
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
            let (server_read, server_write) = split::<IoStream>(IoStream::TcpStream(to_conn));

            tokio::spawn(async move {
                proxy_conn(client_read, server_write, compress).await;
            });
            tokio::spawn(async move {
                proxy_conn(server_read, client_write, compress).await;
            });
        } else {
            eprintln!("failed to connect to {}", to_addr);
        }
    }
}

async fn proxy_conn(
    mut read_conn: ReadHalf<IoStream>,
    mut write_conn: WriteHalf<IoStream>,
    compress: bool,
) {
    let mut buf = vec![0; 1024];

    loop {
        // echo client to server
        match read_conn.read(&mut buf).await {
            Ok(0) => {
                println!("Read connection closed");
                break;
            }
            Ok(n) => {
                let decomp_buf = match compress {
                    true => {
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
                    _ => vec![],
                };

                let write_buffer = match compress {
                    true => &decomp_buf,
                    false => &buf[..n],
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
    use crate::compression::Compressor;
    use crate::iostream::IoStream;
    use crate::reverse_proxy::proxy_conn;
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
    async fn setup_proxy(compress: bool, encrypt: bool) -> TestProxy {
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

        let (in_recv_read, _) = split::<IoStream>(IoStream::TcpStream(in_recv_conn));
        let (_, out_send_write) = split::<IoStream>(IoStream::TcpStream(out_send_conn));

        tokio::spawn(async move {
            proxy_conn(in_recv_read, out_send_write, compress).await;
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

        let mut test_proxy = setup_proxy(false, false).await;

        test_proxy.reader.write_all(&message).await.unwrap();
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, message);
    }

    #[tokio::test]
    async fn proxy_decompressed_content() {
        let message = "Hello world! This is message should be proxied and decompressed.".as_bytes();

        let mut ref_compressor = Compressor::new(Vec::new());
        ref_compressor.write_all(&message).unwrap();
        let compressed_message = ref_compressor.finish().unwrap();

        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(true, false).await;

        test_proxy
            .reader
            .write_all(&compressed_message)
            .await
            .unwrap();
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, message);
    }

    #[tokio::test]
    async fn proxy_large_decompressed_content() {
        // ~2kB message
        let message = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam risus metus, vulputate sed erat non, maximus accumsan augue. Ut eu aliquet urna, sed mollis lectus. Vivamus eu egestas lectus. Donec commodo diam vehicula nisl iaculis, at scelerisque est efficitur. Pellentesque sed dolor arcu. Nullam semper quam risus, quis lobortis sapien mollis vitae. Fusce egestas ante nisl, ac bibendum mi faucibus ac. Phasellus eu libero orci. Cras dignissim in nibh quis eleifend. Duis mattis fermentum nulla ac aliquet. Cras et orci quis erat fermentum auctor et in mauris. Ut ornare, elit a blandit imperdiet, nibh sapien dapibus sapien, non faucibus diam arcu fermentum nunc. Proin feugiat pharetra lectus vitae semper. Fusce sit amet tortor mattis, hendrerit ex nec, iaculis risus.

Nam est nibh, semper sit amet gravida eu, efficitur in tortor. Aenean vel leo vitae enim scelerisque porta at et nibh. Nulla malesuada vel ipsum placerat varius. Aliquam facilisis, dolor quis ultrices condimentum, nisl metus consequat purus, non vulputate odio odio at justo. Fusce rhoncus neque arcu, et venenatis lacus vestibulum at. Nullam tristique tincidunt nunc. Ut mollis sem non turpis accumsan, et volutpat quam suscipit. Cras metus libero, commodo vitae purus vulputate, scelerisque molestie mi. Etiam posuere orci id turpis suscipit egestas. Nunc id faucibus risus.

Duis quis neque sit amet turpis ullamcorper pretium a et turpis. In ultrices eros sit amet odio venenatis varius. Vestibulum id sem iaculis dolor ornare egestas eu sit amet nunc. Integer elit lorem, pretium vestibulum euismod in, imperdiet porttitor nisl. In accumsan elit non rutrum euismod. Integer turpis sem, lobortis non laoreet id, mattis at metus. Sed hendrerit volutpat dui ut consectetur.

Duis efficitur, lacus a condimentum rhoncus, justo ex tristique neque, fermentum imperdiet tortor ex a ante. Mauris a tortor nec sapien volutpat porttitor. Praesent purus erat, viverra sed rhoncus eget, sodales ac felis. Integer scelerisque leo gravida.".as_bytes();

        // Currently the forward proxy separates messages into chunks of at most 1024 bytes
        // TODO: make this maintainable by removing hardcoded values
        let compressed_messages = message.chunks(1024).map(|chunk| {
            let mut ref_compressor = Compressor::new(Vec::new());
            ref_compressor.write_all(chunk).unwrap();
            ref_compressor.finish().unwrap()
        });

        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(true, false).await;

        for msg in compressed_messages {
            test_proxy.reader.write_all(&msg).await.unwrap();
        }
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, message);
    }
}
