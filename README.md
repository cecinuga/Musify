# `Musify`, a p2p platform for sharing and playing music 
### Written in `rust` using `libp2p`.

<img align="center" width="250" height="250" src="./assets/musify.png">

## How it Works?
### Commands
`help` list all command

`ls ps` list all peers in the network.

`sh: <filename>` list all peers that provide the filename.

`dwn: <peer> <filename>` download the file from the peer.

### Working Logic
## Protocol used: 
`mDNS`: used for discovery peers in the network.

`Gossipsub`: used for sending messages between peers.

`Kademlia`: used for providing files's path.

`RequestResponse`: used for request and sending file.