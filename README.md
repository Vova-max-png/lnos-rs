## LNOS-RS - Local Network Overlay System's implementation written in Rust

Experimental overlay networking system implemented in Rust programming language.

## Key concept
### Each node runs a lightweight agent resposible for:
* 📢 Announcing itself in the network
* 📻​ Discovering other nodes
* 📡​​ Maintaining the connection to other nodes
* 📈​ Tracking node availability

## Key features
### Current:
* 🌐 Multicast local nodes discovery
* 📑 Registry of all connected and known nodes
* 🔐 Secure communication using packets signed with signature

### Planned:
* 📝 List of all node's available services
* 🏮 Node's name recognition in commands through hosts file 

## Usage:
### Generate private and public keys
```bash
lnosctl generate-keys
```
#### The keys will be located at /etc/lnos. Do not share your private key for security purposes.

## Examples:
### Examples of node name
```
pc.main.badcoder
laptop.dev.badcoder
pi.router.home
```

### Examples of node resolution
```
ping pc.main.badcoder
ssh laptop.dev.badcoder
```