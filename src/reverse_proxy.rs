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
                    proxy_conn(client_read, server_write, compress, encrypt).await;
                });
                tokio::spawn(async move {
                    proxy_conn(server_read, client_write, compress, encrypt).await;
                });
            } else {
                eprintln!("failed to connect to {}", to_addr);
            }
        }
    })
}

async fn proxy_conn(
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
                println!("Read connection closed");
                break;
            }
            Ok(n) => {
                if let Err(_) = write_conn.write_all(&buf[..n]).await {
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
    use crate::reverse_proxy::proxy_conn;
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

        let (in_recv_read, _) = split::<TcpStream>(in_recv_conn);
        let (_, out_send_write) = split::<TcpStream>(out_send_conn);

        tokio::spawn(async move {
            proxy_conn(in_recv_read, out_send_write, compress, encrypt).await;
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
}
