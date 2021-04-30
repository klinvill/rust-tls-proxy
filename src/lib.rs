pub mod compression;
pub mod forward_proxy;
pub mod reverse_proxy;

pub mod errors {
    error_chain::error_chain! {
        foreign_links {
            NixError(nix::Error);
            IoError(std::io::Error);
            DnsNameError(tokio_rustls::webpki::InvalidDNSNameError);
        }
    }
}
