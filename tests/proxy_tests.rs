use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use rust_tls_proxy::compression::Compressor;
use rust_tls_proxy::{forward_proxy, reverse_proxy};
use std::fs::File;
use std::io::{BufReader, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio_rustls::rustls::internal::pemfile;
use tokio_rustls::rustls::{ClientConfig, NoClientAuth, ServerConfig};
use tokio_rustls::webpki::DNSNameRef;
use tokio_rustls::{TlsAcceptor, TlsConnector};

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
        reverse_proxy::run_async(
            reverse_in_addr,
            vec![reverse_out_addr],
            false,
            false,
            None,
            None,
        )
        .await
        .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, false, false, None)
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
        reverse_proxy::run_async(
            reverse_in_addr,
            vec![reverse_out_addr],
            true,
            false,
            None,
            None,
        )
        .await
        .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, true, false, None)
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

// TODO: these tests are a bunch of hacked together lines. Should refactor out into smaller tests
//  and helper methods.
#[tokio::test]
async fn transparent_compression_proxy_with_large_message() {
    // TODO: hack to make sure previous sockets are freed up
    tokio::time::sleep(Duration::from_secs(2)).await;

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
        reverse_proxy::run_async(
            reverse_in_addr,
            vec![reverse_out_addr],
            true,
            false,
            None,
            None,
        )
        .await
        .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, true, false, None)
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

    assert_eq!(
        forward_out_sent,
        compressed_messages.flatten().collect::<Vec<u8>>()
    );

    let mut reverse_in_conn = TcpStream::connect(reverse_in_addr).await.unwrap();
    let (mut out_recv_conn, _) = out_listener.accept().await.unwrap();
    reverse_in_conn.write_all(&forward_out_sent).await.unwrap();
    reverse_in_conn.shutdown().await.unwrap();
    out_recv_conn.read_to_end(&mut received).await.unwrap();

    assert_eq!(received, message);
}

#[tokio::test]
async fn transparent_encryption_proxy() {
    // TODO: hack to make sure previous sockets are freed up
    tokio::time::sleep(Duration::from_secs(3)).await;

    let message = "Hello world! This is message should be proxied and encrypted.".as_bytes();

    let mut forward_out_sent = Vec::new();
    let mut received = Vec::new();

    let forward_in_addr: SocketAddr = "127.0.0.1:8123".parse().unwrap();
    let forward_out_addr: SocketAddr = "127.0.0.1:9443".parse().unwrap();
    let reverse_in_addr: SocketAddr = "127.0.0.1:8124".parse().unwrap();
    let reverse_out_addr: SocketAddr = "127.0.0.1:8125".parse().unwrap();

    let out_listener = TcpListener::bind(reverse_out_addr).await.unwrap();
    let forward_proxy_listener = TcpListener::bind(forward_in_addr).await.unwrap();
    let forward_out_listener = TcpListener::bind(forward_out_addr).await.unwrap();

    let ca_cert_path = Path::new("tests/certs/ca_cert.pem");
    let cert_path = Path::new("tests/certs/cert.pem");
    let key_path = Path::new("tests/certs/key.pem");

    let certs = pemfile::certs(&mut BufReader::new(File::open(cert_path).unwrap())).unwrap();
    let mut keys =
        pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(key_path).unwrap())).unwrap();
    let key = keys.remove(0);

    let mut server_tls_config = ServerConfig::new(NoClientAuth::new());
    server_tls_config.set_single_cert(certs, key).unwrap();

    let mut client_tls_config = ClientConfig::new();
    client_tls_config
        .root_store
        .add_pem_file(&mut BufReader::new(File::open(ca_cert_path).unwrap()))
        .unwrap();

    tokio::spawn(async move {
        reverse_proxy::run_async(
            reverse_in_addr,
            vec![reverse_out_addr],
            false,
            true,
            Some(cert_path),
            Some(key_path),
        )
        .await
        .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, false, true, Some(ca_cert_path))
            .await
            .unwrap();
    });

    let mut in_send_conn = TcpStream::connect(forward_in_addr).await.unwrap();

    let write_fut = in_send_conn.write_all(&message);

    // Check output from forward proxy to make sure it's compressed
    let (forward_out_tcp_conn, _) = forward_out_listener.accept().await.unwrap();
    let mut forward_out_conn = TlsAcceptor::from(Arc::new(server_tls_config))
        .accept(forward_out_tcp_conn)
        .await
        .unwrap();

    write_fut.await.unwrap();
    in_send_conn.shutdown().await.unwrap();
    forward_out_conn
        .read_to_end(&mut forward_out_sent)
        .await
        .unwrap();

    assert_eq!(forward_out_sent, message);

    let reverse_in_tcp_conn = TcpStream::connect(reverse_in_addr).await.unwrap();
    let connector = TlsConnector::from(Arc::new(client_tls_config));
    let dnsname = DNSNameRef::try_from_ascii_str("localhost").unwrap();
    let mut reverse_in_conn = connector
        .connect(dnsname, reverse_in_tcp_conn)
        .await
        .unwrap();

    let (mut out_recv_conn, _) = out_listener.accept().await.unwrap();
    reverse_in_conn.write_all(&forward_out_sent).await.unwrap();
    reverse_in_conn.shutdown().await.unwrap();
    out_recv_conn.read_to_end(&mut received).await.unwrap();

    assert_eq!(received, message);
}

#[tokio::test]
async fn transparent_encryption_proxy_with_large_message() {
    // TODO: hack to make sure previous sockets are freed up
    tokio::time::sleep(Duration::from_secs(4)).await;

    let message = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam risus metus, vulputate sed erat non, maximus accumsan augue. Ut eu aliquet urna, sed mollis lectus. Vivamus eu egestas lectus. Donec commodo diam vehicula nisl iaculis, at scelerisque est efficitur. Pellentesque sed dolor arcu. Nullam semper quam risus, quis lobortis sapien mollis vitae. Fusce egestas ante nisl, ac bibendum mi faucibus ac. Phasellus eu libero orci. Cras dignissim in nibh quis eleifend. Duis mattis fermentum nulla ac aliquet. Cras et orci quis erat fermentum auctor et in mauris. Ut ornare, elit a blandit imperdiet, nibh sapien dapibus sapien, non faucibus diam arcu fermentum nunc. Proin feugiat pharetra lectus vitae semper. Fusce sit amet tortor mattis, hendrerit ex nec, iaculis risus.

Nam est nibh, semper sit amet gravida eu, efficitur in tortor. Aenean vel leo vitae enim scelerisque porta at et nibh. Nulla malesuada vel ipsum placerat varius. Aliquam facilisis, dolor quis ultrices condimentum, nisl metus consequat purus, non vulputate odio odio at justo. Fusce rhoncus neque arcu, et venenatis lacus vestibulum at. Nullam tristique tincidunt nunc. Ut mollis sem non turpis accumsan, et volutpat quam suscipit. Cras metus libero, commodo vitae purus vulputate, scelerisque molestie mi. Etiam posuere orci id turpis suscipit egestas. Nunc id faucibus risus.

Duis quis neque sit amet turpis ullamcorper pretium a et turpis. In ultrices eros sit amet odio venenatis varius. Vestibulum id sem iaculis dolor ornare egestas eu sit amet nunc. Integer elit lorem, pretium vestibulum euismod in, imperdiet porttitor nisl. In accumsan elit non rutrum euismod. Integer turpis sem, lobortis non laoreet id, mattis at metus. Sed hendrerit volutpat dui ut consectetur.

Duis efficitur, lacus a condimentum rhoncus, justo ex tristique neque, fermentum imperdiet tortor ex a ante. Mauris a tortor nec sapien volutpat porttitor. Praesent purus erat, viverra sed rhoncus eget, sodales ac felis. Integer scelerisque leo gravida.".as_bytes();

    let mut forward_out_sent = Vec::new();
    let mut received = Vec::new();

    let forward_in_addr: SocketAddr = "127.0.0.1:8123".parse().unwrap();
    let forward_out_addr: SocketAddr = "127.0.0.1:9443".parse().unwrap();
    let reverse_in_addr: SocketAddr = "127.0.0.1:8124".parse().unwrap();
    let reverse_out_addr: SocketAddr = "127.0.0.1:8125".parse().unwrap();

    let out_listener = TcpListener::bind(reverse_out_addr).await.unwrap();
    let forward_proxy_listener = TcpListener::bind(forward_in_addr).await.unwrap();
    let forward_out_listener = TcpListener::bind(forward_out_addr).await.unwrap();

    let ca_cert_path = Path::new("tests/certs/ca_cert.pem");
    let cert_path = Path::new("tests/certs/cert.pem");
    let key_path = Path::new("tests/certs/key.pem");

    let certs = pemfile::certs(&mut BufReader::new(File::open(cert_path).unwrap())).unwrap();
    let mut keys =
        pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(key_path).unwrap())).unwrap();
    let key = keys.remove(0);

    let mut server_tls_config = ServerConfig::new(NoClientAuth::new());
    server_tls_config.set_single_cert(certs, key).unwrap();

    let mut client_tls_config = ClientConfig::new();
    client_tls_config
        .root_store
        .add_pem_file(&mut BufReader::new(File::open(ca_cert_path).unwrap()))
        .unwrap();

    tokio::spawn(async move {
        reverse_proxy::run_async(
            reverse_in_addr,
            vec![reverse_out_addr],
            false,
            true,
            Some(cert_path),
            Some(key_path),
        )
        .await
        .unwrap();
    });

    tokio::spawn(async move {
        forward_proxy::forward_proxy(forward_proxy_listener, false, true, Some(ca_cert_path))
            .await
            .unwrap();
    });

    let mut in_send_conn = TcpStream::connect(forward_in_addr).await.unwrap();

    let write_fut = in_send_conn.write_all(&message);

    // Check output from forward proxy to make sure it's compressed
    let (forward_out_tcp_conn, _) = forward_out_listener.accept().await.unwrap();
    let mut forward_out_conn = TlsAcceptor::from(Arc::new(server_tls_config))
        .accept(forward_out_tcp_conn)
        .await
        .unwrap();

    write_fut.await.unwrap();
    in_send_conn.shutdown().await.unwrap();
    forward_out_conn
        .read_to_end(&mut forward_out_sent)
        .await
        .unwrap();

    assert_eq!(forward_out_sent, message);

    let reverse_in_tcp_conn = TcpStream::connect(reverse_in_addr).await.unwrap();
    let connector = TlsConnector::from(Arc::new(client_tls_config));
    let dnsname = DNSNameRef::try_from_ascii_str("localhost").unwrap();
    let mut reverse_in_conn = connector
        .connect(dnsname, reverse_in_tcp_conn)
        .await
        .unwrap();

    let (mut out_recv_conn, _) = out_listener.accept().await.unwrap();
    reverse_in_conn.write_all(&forward_out_sent).await.unwrap();
    reverse_in_conn.shutdown().await.unwrap();
    out_recv_conn.read_to_end(&mut received).await.unwrap();

    assert_eq!(received, message);
}
