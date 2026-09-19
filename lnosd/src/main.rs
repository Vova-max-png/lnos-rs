mod provider;
mod crypto;

use std::net::Ipv4Addr;

use provider::*;

fn main() {
    let provider = Provider::new(Ipv4Addr::new(
        239, 255, 255, 250), 
    8080, 
    2000, 
    2);

    let send_handle = provider.send();
    let receive_handle = provider.receive();
    let nodes_handle = provider.handle_nodes();
    let log_handle = provider.log();

    send_handle.join().unwrap();
    receive_handle.join().unwrap();
    nodes_handle.join().unwrap();
    log_handle.join().unwrap();
}

