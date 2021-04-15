mod compression;
mod forward_proxy;
mod reverse_proxy;

use clap::{Arg, App, SubCommand, AppSettings};

const APP_NAME : &str = "Rust TLS Proxy";
const ABOUT_STR : &str = "Project for network systems class to build a \
                          transport TLS proxy in Rust to encrypt \
                          unencrypted messages";

fn main() {
    let m = App::new(APP_NAME)
            .about(ABOUT_STR)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .arg(Arg::with_name("compress")
                 .short("c")
                 .long("compress")
                 .help("enable compression"))
            .arg(Arg::with_name("encrypt")
                 .short("e")
                 .long("encrypt")
                 .help("enable encryption"))
            .subcommands( vec![
                SubCommand::with_name("forward")
                            .about("start in foward proxy server mode")
                            .arg(Arg::with_name("port")
                                 .short("p")
                                 .long("port")
                                 .help("port number receiving intercepted client connections, default 8080")),
                SubCommand::with_name("reverse")
                            .about("start in reverse proxy server mode")
                            .arg(Arg::with_name("port")
                                 .short("p")
                                 .long("port")
                                 .help("port number receiving incoming connections, default 443"))
                            .arg(Arg::with_name("SERVER")
                                 .help("server addresses in format ip:port")
                                 .required(true)
                                 .multiple(true))])
            .get_matches();

    if let Some(cmd) = m.subcommand_name() {
        match cmd {
            "forward" => println!("forward subcommand"),
            "reverse" => println!("reverse subcommand"),
            _ => panic!("unknown subcommand, clap parser failure"),
        }
    }


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
