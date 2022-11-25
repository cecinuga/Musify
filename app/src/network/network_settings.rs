pub mod network{
    use std::{iter, time::Duration};

    use futures::Sink;
    use libp2p::{Swarm, tcp::{TokioTcpTransport, GenTcpConfig}, core::upgrade, noise::NoiseAuthenticated, mplex::MplexConfig, kad::{store::MemoryStore, Kademlia}, mdns::{Mdns, MdnsConfig}, request_response::{RequestResponse, ProtocolSupport}, gossipsub::{GossipsubConfigBuilder, ValidationMode, Gossipsub, MessageAuthenticity, Topic, IdentTopic}, swarm::SwarmBuilder, identity::Keypair, PeerId};
    use once_cell::sync::Lazy;
    use crate::network::network_behaviour::behaviour::{MyBehaviour, FileExchangeProtocol, FileExchangeCodec};
    use libp2p::Transport;

    pub static KEYS: Lazy<Keypair> = Lazy::new(Keypair::generate_ed25519);
    pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from_public_key(&KEYS.public()));
    pub static TOPIC: Lazy<IdentTopic> = Lazy::new(|| Topic::new("musify"));
    pub static PATH: Lazy<String> = Lazy::new(|| String::from("./assets/music/"));

    pub fn create_swarm()-> Swarm<MyBehaviour> {
        let swarm = {
            let transport = TokioTcpTransport::new(GenTcpConfig::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(
                NoiseAuthenticated::xx(&KEYS)
                    .expect("Signing libp2p-nois static DH keypair failed"),
            )
            .multiplex(MplexConfig::new())
            .boxed();
            
            let mdns = Mdns::new(MdnsConfig::default()).unwrap();
            let store = MemoryStore::new(PEER_ID.clone());
            let kademlia = Kademlia::new(PEER_ID.clone(), store);
            let request_response = RequestResponse::new(
                FileExchangeCodec(),
                iter::once((FileExchangeProtocol(), ProtocolSupport::Full)),
                Default::default(),
            );

            let gossipsub_config = GossipsubConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(ValidationMode::Strict)
                .build()
                .expect("Valid config");     
            let mut gossipsub: Gossipsub = Gossipsub::new(MessageAuthenticity::Signed(KEYS.clone()), gossipsub_config).expect("Correct configuration");

            gossipsub.subscribe(&TOPIC).unwrap();

            let behaviour = MyBehaviour{request_response, kademlia, gossipsub, mdns };
            SwarmBuilder::new(transport, behaviour, PEER_ID.clone())
                .executor(Box::new(|fut|{
                    tokio::spawn(fut);
                }))
                .build()
        };swarm
    }
}