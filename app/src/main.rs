use tokio;
use libp2p::{
    mdns::{Mdns, MdnsConfig},
    identity::{
        Keypair
    },
    PeerId,
    gossipsub::IdentTopic as Topic,
};
use std::error::Error;
use once_cell::sync::Lazy;
use app::{behaviour};


static KEYS: Lazy<Keypair> = Lazy::new(Keypair::generate_ed25519);
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("musify"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("PeerID: {}", PEER_ID.clone());

    let mut swarm = {
        let mdns = Mdns::new(MdnsConfig::default())?;
    };

    Ok(())
}