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

mod forward_proxy;
mod reverse_proxy;

use error_chain::bail;
use error_chain::ChainedError;
mod errors {
    error_chain::error_chain! {
        foreign_links {
            NixError(nix::Error);
        }
    }
}
use errors::*;

use clap::{App, AppSettings, Arg, SubCommand};
use std::net::{IpAddr, SocketAddr};

enum ServerSettings {
    Forward {
        addr: SocketAddr,
    },
    Reverse {
        addr: SocketAddr,
        server_ips: Vec<SocketAddr>,
    },
}

const APP_NAME: &str = "Rust TLS Proxy";
const ABOUT_STR: &str = "Project for network systems class to build a \
    transport TLS proxy in Rust to encrypt unencrypted messages";

const FORWARD_PORT_HELP: &str = const_format::formatcp!(
    "port number receiving intercepted client connections, default {}",
    forward_proxy::PROXY_REDIR_PORT
);

const REVERSE_PORT_HELP: &str = const_format::formatcp!(
    "port number receiving incoming connections, default {}",
    reverse_proxy::HTTPS_PORT
);

fn run() -> Result<()> {
    let m = App::new(APP_NAME)
        .about(ABOUT_STR)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("compress")
                .short("c")
                .long("compress")
                .help("enable compression"),
        )
        .arg(
            Arg::with_name("encrypt")
                .short("e")
                .long("encrypt")
                .help("enable encryption"),
        )
        .subcommands(vec![
            SubCommand::with_name("forward")
                .about("start in foward proxy server mode")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .help(FORWARD_PORT_HELP)
                        .takes_value(true),
                ),
            SubCommand::with_name("reverse")
                .about("start in reverse proxy server mode")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .help(REVERSE_PORT_HELP)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("SERVERS")
                        .help("server addresses in format ip:port")
                        .required(true)
                        .multiple(true),
                ),
        ])
        .get_matches_safe()
        .chain_err(|| "error parsing arguments")?;

    let compress = m.is_present("compress");

    let encrypt = m.is_present("encrypt");

    let server = match m.subcommand() {
        ("forward", Some(sub_m)) => ServerSettings::Forward {
            addr: {
                let port = match sub_m.value_of("port") {
                    Some(p) => p
                        .parse()
                        .chain_err(|| format!("error parsing port number \"{}\"", p))?,
                    None => forward_proxy::PROXY_REDIR_PORT,
                };

                SocketAddr::from((IpAddr::from([0, 0, 0, 0]), port))
            },
        },

        ("reverse", Some(sub_m)) => ServerSettings::Reverse {
            addr: {
                let port = match sub_m.value_of("port") {
                    Some(p) => p
                        .parse()
                        .chain_err(|| format!("error parsing port number \"{}\"", p))?,
                    None => reverse_proxy::HTTPS_PORT,
                };

                SocketAddr::from((IpAddr::from([0, 0, 0, 0]), port))
            },

            server_ips: match sub_m.values_of("SERVERS") {
                Some(addrs) => addrs
                    .map(|a| {
                        a.parse::<SocketAddr>()
                            .chain_err(|| format!("error parsing socket address \"{}\"", a))
                    })
                    .collect::<Result<_>>()?,
                None => bail!("no server addreses"),
            },
        },

        _ => bail!("unknown subcommand"),
    };

    return match server {
        ServerSettings::Forward { addr } => forward_proxy::run(addr, compress, encrypt)
            .chain_err(|| "error in forward_proxy::run()"),

        ServerSettings::Reverse { addr, server_ips } => {
            reverse_proxy::run(addr, server_ips, compress, encrypt)
                .chain_err(|| "error in reverse_proxy::run()")
        }
    };
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e.display_chain().to_string());
        std::process::exit(1);
    }
}
