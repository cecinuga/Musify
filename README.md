# `Musify`, a p2p platform for sharing and playing music 
### Written in `rust` using `libp2p`.

![Musify](relative=assets/musify.png?raw=true "Musify")

## How it Works?
### Commands
`ls ps` list all peers in the network.

### Working Logic
## Protocol used: 
`mDND`: used for discovery peers in the network.

`Gossipsub`: used for sending messages between peers.

`Kademlia`: used for providing files's path.

`RequestResponse`: used for request and sending file.