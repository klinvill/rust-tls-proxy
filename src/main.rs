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

mod compression;
mod forward_proxy;
mod reverse_proxy;

use clap::{Arg, App, SubCommand, AppSettings};
use const_format::formatcp;
use std::error::Error;
use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};

enum Server {
    Forward { port: u16 },
    Reverse { port: u16, server_ips: Vec<SocketAddrV4> },
}

const APP_NAME : &str = "Rust TLS Proxy";
const ABOUT_STR : &str =
    "Project for network systems class to build a transport TLS proxy in Rust \
    to encrypt unencrypted messages";

const FORWARD_PORT_HELP : &str = formatcp!(
    "port number receiving intercepted client connections, default {}", 
    forward_proxy::DEFAULT_PORT);

const REVERSE_PORT_HELP : &str = formatcp!(
    "port number receiving incoming connections, default {}",
    reverse_proxy::DEFAULT_PORT);

fn run() -> Result<(), Box<Error>> {
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
                     .help(FORWARD_PORT_HELP)
                     .takes_value(true)),
                SubCommand::with_name("reverse")
                .about("start in reverse proxy server mode")
                .arg(Arg::with_name("port")
                     .short("p")
                     .long("port")
                     .help(REVERSE_PORT_HELP)
                     .takes_value(true))
                .arg(Arg::with_name("SERVERS")
                     .help("server addresses in format ip:port")
                     .required(true)
                     .multiple(true))])
            .get_matches_safe()?;

    let compress = m.is_present("compress");
    let encrypt = m.is_present("encrypt");
    let server = match m.subcommand() {
        ("forward", Some(sub_m)) => Server::Forward {
            port: match sub_m.value_of("port") {
                Some(p) => p.parse()?,
                None => forward_proxy::DEFAULT_PORT,
            },
        },
        ("reverse", Some(sub_m)) => Server::Reverse {
            port: match sub_m.value_of("port") {
                Some(p) => p.parse()?,
                None => reverse_proxy::DEFAULT_PORT,
            },
            server_ips: match sub_m.values_of("SERVERS") {
                Some(addrs) => addrs.map(|a| a.parse::<SocketAddrV4>()) 
                    .collect::<Result<Vec<_>, _>>()?,
                None => return Err("no server addreses".into()),
            },
        },
        _ => return Err("unknown subcommand".into()),
    };

    let local_ip = "127.0.0.1".parse::<Ipv4Addr>()?;
    let local_addr = SocketAddrV4::new(local_ip, match server {
        Server::Forward{port} => port,
        Server::Reverse{port, ..} => port,
    });
    let listen_socket = TcpListener::bind(local_addr)?;

    match server {
        Server::Forward{..} => forward_proxy::run(listen_socket, compress, encrypt)?,
        Server::Reverse{server_ips, ..} => reverse_proxy::run(listen_socket, server_ips, compress, encrypt)?,
    }

    return Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
