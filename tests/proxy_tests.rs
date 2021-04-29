use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use rust_tls_proxy::compression::Compressor;
use rust_tls_proxy::{forward_proxy, reverse_proxy};
use std::io::Write;
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
        reverse_proxy::run_async(reverse_in_addr, vec![reverse_out_addr], false, false)
            .await
            .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, false, false)
            .await
            .unwrap();
    });

    let mut in_send_conn = TcpStream::connect(forward_in_addr).await.unwrap();

    in_send_conn.write_all(&message).await.unwrap();

    let (mut out_recv_conn, _) = out_listener.accept().await.unwrap();

    in_send_conn.shutdown().await.unwrap();
    out_recv_conn.read_to_end(&mut received).await.unwrap();

    assert_eq!(received, message);
}

// TODO: these tests are a bunch of hacked together lines. Should refactor out into smaller tests
//  and helper methods.
#[tokio::test]
async fn transparent_compression_proxy() {
    // TODO: hack to make sure previous sockets are freed up
    tokio::time::sleep(Duration::from_secs(1)).await;

    let message = "Hello world! This is message should be proxied.".as_bytes();
    let mut ref_compressor = Compressor::new(Vec::new());
    ref_compressor.write_all(&message).unwrap();
    let compressed_message = ref_compressor.finish().unwrap();

    let mut forward_out_sent = Vec::new();
    let mut received = Vec::new();

    let forward_in_addr: SocketAddr = "127.0.0.1:8123".parse().unwrap();
    let forward_out_addr: SocketAddr = "127.0.0.1:9443".parse().unwrap();
    let reverse_in_addr: SocketAddr = "127.0.0.1:8124".parse().unwrap();
    let reverse_out_addr: SocketAddr = "127.0.0.1:8125".parse().unwrap();

    let out_listener = TcpListener::bind(reverse_out_addr).await.unwrap();
    let forward_proxy_listener = TcpListener::bind(forward_in_addr).await.unwrap();
    let forward_out_listener = TcpListener::bind(forward_out_addr).await.unwrap();

    tokio::spawn(async move {
        reverse_proxy::run_async(reverse_in_addr, vec![reverse_out_addr], true, false)
            .await
            .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, true, false)
            .await
            .unwrap();
    });

    let mut in_send_conn = TcpStream::connect(forward_in_addr).await.unwrap();

    let write_fut = in_send_conn.write_all(&message);

    // Check output from forward proxy to make sure it's compressed
    let (mut forward_out_conn, _) = forward_out_listener.accept().await.unwrap();
    write_fut.await.unwrap();
    in_send_conn.shutdown().await.unwrap();
    forward_out_conn
        .read_to_end(&mut forward_out_sent)
        .await
        .unwrap();

    assert_eq!(forward_out_sent, compressed_message);

    let mut reverse_in_conn = TcpStream::connect(reverse_in_addr).await.unwrap();
    let (mut out_recv_conn, _) = out_listener.accept().await.unwrap();
    reverse_in_conn.write_all(&forward_out_sent).await.unwrap();
    reverse_in_conn.shutdown().await.unwrap();
    out_recv_conn.read_to_end(&mut received).await.unwrap();

    assert_eq!(received, message);
}
