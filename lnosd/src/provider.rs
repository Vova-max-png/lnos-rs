use core::fmt;
use std::net::{UdpSocket, Ipv4Addr};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use serde::{Deserialize, Serialize};

use crate::crypto::Crypto;

pub struct Provider {
    sock: Arc<UdpSocket>,
    mcast_group: Ipv4Addr,
    port: u16,
    known_nodes: Arc<Mutex<Vec<Node>>>,
    just_started: bool,
    crypto: Arc<Mutex<Crypto>>,
    logs_duration: u64,
    node_timeout: u64
}

impl Provider {
    pub fn new(mcast_group: Ipv4Addr, port: u16, logs_duration: u64, node_timeout: u64) -> Self {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap();

        socket.set_multicast_ttl_v4(1).unwrap();
        socket.set_multicast_loop_v4(true).unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

        socket.join_multicast_v4(&mcast_group, &Ipv4Addr::UNSPECIFIED).unwrap();

        Self {
            sock: Arc::new(socket),
            mcast_group,
            port,
            known_nodes: Arc::new(Mutex::new(Vec::new())),
            just_started: true,
            crypto: Arc::new(Mutex::new(Crypto::new())),
            logs_duration,
            node_timeout
        }
    }

    // Send is responsible for sending packets to the broadcast group
    // If it stops sending packets other clients will mark current node offline
    pub fn send(&self) -> JoinHandle<()> {
        let sender_socket = Arc::clone(&self.sock);

        let group = self.mcast_group;
        let port = self.port;
        let mut announce = self.just_started;

        let crypto = Arc::clone(&self.crypto);

        let sender_handle = thread::spawn(move || {
            loop {
                let (public_key, signature) = {
                    let crypto_guard = crypto.lock().unwrap();
                    let pub_key = if announce { Some(crypto_guard.public_key.clone()) } else { None };
                    let sign = crypto_guard.sign("pc.main.badcoder".as_bytes()).to_bytes().to_vec();
                    (pub_key, sign)
                };
                
                let packet = Packet {
                    name: "pc.main.badcoder".to_string(),
                    public_key,
                    signature,
                    announce
                };
                let data = serde_json::to_vec(&packet).unwrap();

                if announce { announce = false; }

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

    // Receiver handles all the incoming packets from the broadcast group
    // It's responsible for receiving, processing and saving the data from packet
    pub fn receive(&self) -> JoinHandle<()> {
        let receiver_socket = Arc::clone(&self.sock);

        let known_nodes = Arc::clone(&self.known_nodes);
        let crypto = Arc::clone(&self.crypto);

        let receiver_handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {  
                match receiver_socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        let valid_bytes = &buf[..len];

                        match serde_json::from_slice::<Packet>(&valid_bytes) {
                            Ok(packet) => {
                                let mut known_nodes_guard = known_nodes.lock().unwrap();

                                let parsed_src_addr = Ipv4Addr::from_str(&src.to_string().split(':').collect::<Vec<&str>>()[0]).unwrap();
                                
                                if packet.announce && 
                                    known_nodes_guard.iter().find(|x| x.addr == parsed_src_addr).is_none() {
                                    let new_node = Node {
                                        name: packet.name.clone(),
                                        status: NodeStatus::Online,
                                        addr: parsed_src_addr,
                                        public_key: packet.public_key.clone().unwrap(),
                                        last_seen: Instant::now()
                                    };
                                    known_nodes_guard.push(new_node);
                                }

                                let node: &mut Node = known_nodes_guard.iter_mut().find(|x| x.addr == parsed_src_addr).expect(format!("Couldn't find node with {} addr", src.to_string()).as_str());

                                match crypto.lock().unwrap().verify(node.public_key.as_bytes(), &packet.signature, packet.name.as_bytes()) {
                                    true => {},
                                    false => {
                                        eprintln!("Couldn't verify packet from {}!", node.addr.to_string());
                                        continue;
                                    }
                                }
                                node.last_seen = Instant::now();

                                // for node in known_nodes_guard.iter() {
                                //     println!("{}", node);
                                // }
                            },
                            Err(e) => {
                                eprintln!("[ERROR] Failed to parse packet: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        }
                        eprintln!("[ERROR] recv_from failed: {}", e);
                    }
                }
            }
        });

        receiver_handle
    }

    // Log stands for logging all the current nodes connected to the broadcast group
    // Optional function, call it when needed
    pub fn log(&self) -> JoinHandle<()> {
        let known_nodes = Arc::clone(&self.known_nodes);
        let logs_duration = Duration::from_millis(self.logs_duration);

        let log_handler = thread::spawn(move || {
            loop {
                {
                    let known_nodes = known_nodes.lock().unwrap();

                    for node in known_nodes.iter() {
                        println!("{}", node);
                    }
                }

                thread::sleep(logs_duration);
            }
        });

        log_handler
    }

    // Handle nodes is responsible for monitoring the nodes
    // It changes their status according to the their state
    pub fn handle_nodes(&self) -> JoinHandle<()> {
        let known_nodes = Arc::clone(&self.known_nodes);
        let node_timeout = self.node_timeout;

        let nodes_handler = thread::spawn(move || {
            {
                let mut nodes = known_nodes.lock().unwrap();
                for node in nodes.iter_mut() {
                    if node.last_seen.elapsed().as_secs() >= node_timeout {
                        node.status = NodeStatus::Offline;
                    } else {
                        node.status = NodeStatus::Online;
                    }
                }
            }
        });

        nodes_handler
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    announce: bool,
    signature: Vec<u8>,
    public_key: Option<String>,
    name: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    name: String,
    status: NodeStatus,
    public_key: String,
    addr: Ipv4Addr,
    #[serde(skip, default = "current_instant")]
    last_seen: Instant
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            NodeStatus::Online => {
                write!(f,
                    "Node '{}' [{}] at {}", 
                    self.name, self.status, self.addr)
            },
            _ => {
                write!(f,
                    "Node '{}' [{}] at {} last seen {} secs ago", 
                    self.name, self.status, self.addr, self.last_seen.elapsed().as_secs())
            }
        }
    }
}

fn current_instant() -> Instant {
    Instant::now()
}

#[derive(Serialize, Deserialize, Debug)]
enum NodeStatus {
    Online = 1,
    Offline = 0
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            NodeStatus::Online => "Online",
            NodeStatus::Offline => "Offline"
        })
    }
}