use tokio::{io::{BufReader, stdin, AsyncBufReadExt}, select};
use libp2p::{
    futures::StreamExt,
    core::upgrade,
    Transport,
    swarm::{SwarmBuilder,SwarmEvent, handler::multi},
    kad::{record::store::MemoryStore, Kademlia},
    noise::{NoiseAuthenticated},
    mplex::{MplexConfig},
    tcp::GenTcpConfig,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    identity::{
        Keypair
    },
    PeerId,
    gossipsub::{IdentTopic as Topic, GossipsubConfigBuilder,MessageAuthenticity, ValidationMode, Gossipsub}, tcp::TokioTcpTransport,
    request_response::{
        ProtocolSupport, RequestResponse, 
    },
};
use std::{error::Error, iter, time::Duration};
use once_cell::sync::Lazy;
use app::{network::behaviour::behaviour::{
    FileExchangeProtocol, 
    FileExchangeCodec, 
    MyBehaviour, 
    MyBehaviourEvent
}, handle_command};


static KEYS: Lazy<Keypair> = Lazy::new(Keypair::generate_ed25519);
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("musify"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("PeerID: {}", PEER_ID.clone());
    
    let mut stdin = BufReader::new(stdin()).lines();

    let mut swarm = {
        let transport = TokioTcpTransport::new(GenTcpConfig::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            NoiseAuthenticated::xx(&KEYS)
                .expect("Signing libp2p-nois static DH keypair failed"),
        )
        .multiplex(MplexConfig::new())
        .boxed();
        
        let mdns = Mdns::new(MdnsConfig::default())?;
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
    };
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        select!{
            line = stdin.next_line() => handle_command(&mut swarm, &line.unwrap().expect("Message not sended.")).await,
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr{ address, .. } => {
                    println!("{}", address)
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm.behaviour_mut().request_response.add_address(&peer_id, multiaddr.clone());
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                    }
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        swarm.behaviour_mut().request_response.remove_address(&peer_id, &multiaddr);
                        swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
                    }
                }
                _=>{}
            }
        }
    }

    Ok(())
}