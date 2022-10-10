use std::net::UdpSocket;

use dnsserv::dns::interface::handle_inbound_query;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let sock = UdpSocket::bind(("0.0.0.0", 2053))?;

    loop {
        match handle_inbound_query(&sock) {
            Ok(_)  => {},
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
