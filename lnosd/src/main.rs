mod provider;
use std::net::Ipv4Addr;

use provider::*;

fn main() {
    let provider = Provider::new(Ipv4Addr::new(239, 255, 255, 250), 8080);

    let listen_handle = provider.send();
    let receive_handle = provider.receive();

    listen_handle.join().unwrap();
    receive_handle.join().unwrap();
}

