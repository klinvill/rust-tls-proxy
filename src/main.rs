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
use std::io;
use std::net::{TcpListener, Ipv4Addr, SocketAddrV4, Shutdown};
use std::process;

enum ServerType {
    Forward,
    Reverse,
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
            .get_matches_safe();

    let m = match m {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(exitcode::USAGE);
        }
    };

    let compress = m.is_present("compress");
    let encrypt = m.is_present("encrypt");
    let server = match m.subcommand_name() {
        Some("forward") => ServerType::Forward,
        Some("reverse") => ServerType::Reverse,
        Some(n) => {
            eprintln!("unknown subcommand \"{}\"", n);
            process::exit(exitcode::USAGE);
        },
        None => {
            eprintln!("\"forward\" or \"reverse\" subcommand needed");
            process::exit(exitcode::USAGE);
        },
    };
    let port = match m.subcommand() {
        (_, Some(sub_m)) => match sub_m.value_of("port") {
            Some(p) => p.parse().unwrap_or_else(|e| {
                eprintln!("error parsing port \"{}\": {}", p, e);
                process::exit(exitcode::DATAERR);
            }),
            None => match server {
                ServerType::Forward => forward_proxy::DEFAULT_PORT, 
                ServerType::Reverse => reverse_proxy::DEFAULT_PORT, 
            },
        },
        _ => {
            eprintln!("socket port number needed");
            process::exit(exitcode::USAGE);
        }
    };
    let server_ips: Option<Vec<SocketAddrV4>> = match m.subcommand() {
        ("reverse", Some(sub_m)) => Some(sub_m.values_of("SERVERS")
            .unwrap_or_else(|| {
                eprintln!("reverse proxy needs at least one server ip address");
                process::exit(exitcode::USAGE);
            })
            .map(|s| s.parse::<SocketAddrV4>().unwrap_or_else(|e| {
                eprintln!("Error parsing ip address from \"{}\": {}", s, e);
                process::exit(exitcode::DATAERR);
            }))
            .collect()),
        _ => None,
    };
    let listen_socket = open_listen_socket(port)
        .unwrap_or_else(|e| {
        eprintln!("failed to open socket on port {}: {}", port, e);
        process::exit(exitcode::OSERR);
    });

    match server {
        ServerType::Forward => forward_proxy::run(listen_socket, compress, encrypt)
            .unwrap_or_else(|e| {
                eprintln!("error in forward_proxy::run(): {}", e);
                process::exit(exitcode::OSERR);
            }),
        ServerType::Reverse => {
            let server_ips = server_ips.unwrap_or_else(|| {
                eprintln!("no server ip addresses");
                process::exit(exitcode::USAGE);
            });
            reverse_proxy::run(listen_socket, server_ips, compress, encrypt)
                .unwrap_or_else(|e| {
                    eprintln!("error in forward_proxy::run(): {}", e);
                    process::exit(exitcode::OSERR);
                });
        },
    }

    process::exit(exitcode::OK);
}

fn open_listen_socket(port : u16) -> io::Result<TcpListener> {
    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    TcpListener::bind(socket)
}
