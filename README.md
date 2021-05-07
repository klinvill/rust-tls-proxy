# rust-tls-proxy
Project for network systems class to build a transport TLS proxy in Rust to encrypt unencrypted messages

Welcome to the rust-tls-proxy wiki!

#### Routing:  
* Expect HTTP to use port 9980.  
* Client router sends destination port 9980 (HTTP port) to port 8080 (using mangle table).  
* Forward Proxy (on client router) listens on 8080 and changes outgoing port to 9443 (HTTPS port) if using encryption.  
* Reverse proxy sits on 170.40.17.19 and redirects traffic to 170.40.17.10 (server's ip) and port 9443 to the proxy application port (8080). There is no kernel-level redirection (iptables, mangle table, etc), so the reverse proxy will only activate if the client uses it as the destination address (where the reverse proxy's socket will listen).  
* Expect server to use port 9443.  

#### Test client and server: "forum" program
There are an "example client" and "example forum" in examples/forum. Run them with no arguments with python3 to see the options. In HTTPS mode, it is possible but not advised to forego certificate verification (similar to --insecure on curl). This insecure mode is possible with example_client.py but not with the Rust proxies.

#### Different use cases:
1. Run both client and server on HTTPS port (9443) (proxies do not interfere). Use the cert /home/ubuntu/certs/ca_cert.pem  
2. Run client on HTTP port (9980), use forward proxy with encryption, run server with HTTPS.  
3. Run client on HTTP port (9980) (destination ip server-proxy), run both proxies with encryption, run server with HTTP on port 9443.

#### What doesn't work:  
1. Running HTTP client (destination ip = reverse proxy), forward proxy with encryption, reverse proxy without encryption, and HTTPS server

#### Running forward proxy:
sudo target/debug/rust_tls_proxy forward -e --root-cert /home/ubuntu/certs/ca_cert.pem
opening listener socket on 0.0.0.0:8080  
#### Running reverse proxy:
target/debug/rust_tls_proxy reverse --cert-chain /home/ubuntu/certs/server-router-cert.pem --key /home/ubuntu/certs/server-router-key.pem 172.40.17.10:8080  


#### Code: 
The Rust code is as follows:
1. io is asynchronous using tokio::io::poll_read / poll_write, which will not block the caller if the buffer is not ready. This is modified to only work with TcpStream and TlsStream.  
2.  Forward proxy changes destination port to reverse_proxy::HTTPS_PORT (port 9443) (it seems like it might not always be doing this). It also uses a transparent socket (I'm not sure what that means). If using encryption, the forward proxy creates a TlsStream and must verify that the domain name it is connecting to matches the certificate. It does not seem to have error handling / chaining for a failed TLS connection. The forward proxy uses tokio::spawn to create two "async" (threads): one for forwarding from client to server, and one from forwarding responses from server to client.
3. Reverse proxy: Listens on port 9443 by default. 

#### Building the code:
Run `cargo build` to build the binaries for development, and `cargo build --release` to build the optimized binaries for benchmarking and release. You can also run the tests using the `cargo test` command.

#### Encryption requirements:
Running the forward proxy with encryption requires a trusted ca cert be provided to validate received certs. Running the reverse proxy with encryption requires a signed cert and key.

#### Compression requirements:
The compression layer is a custom layer, therefore the compression messages won't be properly interpreted unless the receiver also accepts our custom compression scheme. As a result, we recommend only using compression when using both the forward and reverse proxies with compression enabled.
