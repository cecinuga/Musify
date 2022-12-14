use tokio::{io::{BufReader, stdin, AsyncBufReadExt}, select};
use libp2p::{
    futures::StreamExt,
    swarm::{SwarmEvent,},
    mdns::{MdnsEvent}, kad::{KademliaEvent, QueryResult}, request_response::{RequestResponseEvent, RequestResponseMessage},
};
use std::{error::Error,};

use app::{
    network::network_behaviour::behaviour::{
        MyBehaviourEvent
    }, 
    network::{network_settings::network::{create_swarm, PEER_ID, PATH}, network_behaviour::behaviour::FileResponse},
    handle_command, providing_files, request_download, response_download, save_file, 
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let mut stdin = BufReader::new(stdin()).lines();
    let mut swarm = create_swarm();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("[#]PeerID: {}", PEER_ID.clone());
    providing_files(&mut swarm.behaviour_mut().kademlia).await?;
    println!("\t");
    loop {
        select!{
            line = stdin.next_line() => handle_command(&mut swarm, &line.unwrap().expect("Message not sended.")).await,
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr{ address, .. } => {
                    println!("{}", address)
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::RequestResponse(RequestResponseEvent::Message{ message, ..},
                )) => match message {
                    RequestResponseMessage::Request{
                        request, channel, ..
                    } => { 
                        response_download(&mut swarm, channel, request);
                    }
                    RequestResponseMessage::Response{ 
                        request_id, response 
                    } => {
                        save_file(&mut swarm, response);
                    }
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(KademliaEvent::OutboundQueryCompleted{result, ..})) => {
                    match result {
                        QueryResult::GetProviders(Ok(ok)) => {
                            if(!ok.providers.is_empty()) {
                                println!("{:?}", ok.providers);
                            } else{println!("[#]Your item is not founded.")}
                        }
                        _=>{}
                    }
                }
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
}