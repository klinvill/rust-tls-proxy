mod compression;

fn main() {
    // General flow:
    //  - parse command line arguments:
    //      - run as forward or reverse proxy depending on args
    //      - optionally compress data before encrypting (vulnerable to CRIME-style attacks)
    //  - accept new TCP connections, handle each one in a new thread
    //
    //  - If forward proxy (runs for each new connection):
    //      - Check to see if payload already includes TLS records, if so, forward the packet without modification
    //      - Create a new TLS client session (assuming the use of rustls)
    //      - Create a new TCP connection to the target server
    //      - Loop until either client or server connection closes:
    //          - If data received from client, write to server using write() and then write_tls() (assuming rustls)
    //              - Optionally compress data before sending
    //          - If data received from server, read the data using read_tls() and then read(), then send to client (assuming rustls)
    //
    //  - If reverse proxy (runs for each new connection):
    //      - Check to see if payload includes TLS records, if not, forward the packet without modification
    //      - Create a new TLS server session (assuming rustls)
    //      - Create a new TCP connection to the target server
    //      - Loop until either client or server connection closes:
    //          - If data received from client, read the data using read_tls() and then read(), then send to server (assuming rustls)
    //          - If data received from server, send to client using write() and then write_tls() (assuming rustls)
    //              - Optionally compress data before sending
}
