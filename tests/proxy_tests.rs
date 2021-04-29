use tokio;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use rust_tls_proxy::{forward_proxy, reverse_proxy};
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::test]
async fn transparent_proxy() {
    let message = "Hello world! This is message should be proxied.".as_bytes();
    let mut received = Vec::new();

    let forward_in_addr: SocketAddr = "127.0.0.1:8123".parse().unwrap();
    let reverse_in_addr: SocketAddr = "127.0.0.1:9443".parse().unwrap();
    let reverse_out_addr: SocketAddr = "127.0.0.1:8125".parse().unwrap();

    let out_listener = TcpListener::bind(reverse_out_addr).await.unwrap();
    let forward_proxy_listener = TcpListener::bind(forward_in_addr).await.unwrap();

    tokio::spawn(async move {
        reverse_proxy::run_async(reverse_in_addr, vec![reverse_out_addr], false, false).await.unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, false, false).await.unwrap();
    });

    let mut in_send_conn = TcpStream::connect(forward_in_addr).await.unwrap();

    in_send_conn.write_all(&message).await.unwrap();

    let (mut out_recv_conn, _) = out_listener.accept().await.unwrap();

    in_send_conn.shutdown().await.unwrap();
    out_recv_conn.read_to_end(&mut received).await.unwrap();

    assert_eq!(received, message);
}
