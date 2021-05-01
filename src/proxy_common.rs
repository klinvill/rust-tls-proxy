use crate::compression::{compress, decompress, split_frames, Direction};
use crate::errors::Result;
use crate::iostream::IoStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};

pub async fn proxy_conn(
    mut read_conn: ReadHalf<IoStream>,
    mut write_conn: WriteHalf<IoStream>,
    compress_direction: Option<Direction>,
) -> Result<()> {
    let mut buf = vec![0; 1024];

    loop {
        // proxy from the read connection to the write connection
        match read_conn.read(&mut buf).await {
            Ok(0) => {
                println!("Read connection closed");
                break;
            }
            Ok(n) => {
                let comp_buf = match compress_direction {
                    Some(Direction::Decompress) => split_frames(&buf[..n])
                        .iter()
                        .map(|frame| decompress(frame))
                        .collect::<std::io::Result<Vec<Vec<u8>>>>()?
                        .into_iter()
                        .flatten()
                        .collect(),
                    Some(Direction::Compress) => compress(&buf[..n])?,
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::compression::{compress, decompress, split_frames, Direction};
    use crate::iostream::IoStream;
    use crate::proxy_common::proxy_conn;
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

        tokio::spawn(
            async move { proxy_conn(in_recv_read, out_send_write, compress_direction).await },
        );

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
        let expected_message = compress(&message).unwrap();

        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(Some(Direction::Compress)).await;

        test_proxy.reader.write_all(&message).await.unwrap();
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, expected_message);
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
            .flat_map(|frame| decompress(frame).unwrap())
            .collect();

        assert_eq!(message, decompressed_data);
    }

    #[tokio::test]
    async fn proxy_decompressed_content() {
        let message = "Hello world! This is message should be proxied and decompressed.".as_bytes();
        let compressed_message = compress(&message).unwrap();

        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(Some(Direction::Decompress)).await;

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
        let compressed_messages = message.chunks(1024).map(|chunk| compress(chunk).unwrap());

        let mut received = Vec::new();

        let mut test_proxy = setup_proxy(Some(Direction::Decompress)).await;

        for msg in compressed_messages {
            test_proxy.reader.write_all(&msg).await.unwrap();
        }
        test_proxy.reader.shutdown().await.unwrap();
        test_proxy.writer.read_to_end(&mut received).await.unwrap();

        assert_eq!(received, message);
    }
}
