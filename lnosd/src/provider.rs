use std::net::{UdpSocket, Ipv4Addr};
use std::time::Duration;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use serde::{Deserialize, Serialize};


pub struct Provider {
    sock: Arc<UdpSocket>,
    mcast_group: Ipv4Addr,
    port: u16,
    known_nodes: Vec<Node>
}

impl Provider {
    pub fn new(mcast_group: Ipv4Addr, port: u16) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();

        socket.set_multicast_ttl_v4(1).unwrap();
        socket.set_multicast_loop_v4(true).unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

        socket.join_multicast_v4(&mcast_group, &Ipv4Addr::UNSPECIFIED).unwrap();

        Self {
            sock: Arc::new(socket),
            mcast_group,
            port,
            known_nodes: Vec::new()
        }
    }

    pub fn send(&self) -> JoinHandle<()> {
        let sender_socket = Arc::clone(&self.sock);

        let group = self.mcast_group;
        let port = self.port;

        let sender_handle = thread::spawn(move || {
            // let msg = b"Announce payload";
            let packet = Packet {
                name: "pc.main.badcoder".to_string(),
                announce: false
            };
            let data = serde_json::to_vec(&packet).unwrap();
            loop {
                if let Err(e) = sender_socket.send_to(&data, format!("{}:{}", group, port)) {
                    eprintln!("[ERROR] send_to failed: {}", e);
                    break;
                }

                println!("[DEBUG] Sent {} bytes!", data.len());
                thread::sleep(Duration::from_millis(2000));
            }
        });

        sender_handle
    }

    pub fn receive(&self) -> JoinHandle<()> {
        let receiver_socket = Arc::clone(&self.sock);
        let receiver_handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {  
                match receiver_socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        let valid_bytes = &buf[..len];

                        match serde_json::from_slice::<Packet>(&valid_bytes) {
                            Ok(packet) => {
                                println!("[DEBUG] Received {} bytes from {}: {:#?}", len, src, packet);
                            },
                            Err(e) => {
                                eprintln!("[ERROR] Failed to parse packet: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            continue;;
                        }
                        eprintln!("[ERROR] recv_from failed: {}", e);
                    }
                }
            }
        });

        receiver_handle
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    announce: bool,
    name: String
}

struct Node {
    name: String,
    status: NodeStatus,
    addr: Ipv4Addr
}

enum NodeStatus {
    Online = 1,
    Offline = 0
}