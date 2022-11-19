use libp2p::{Swarm};
use tokio::io::{Lines, BufReader, Stdin};
use network::network_behaviour;

pub async fn handle_command(swarm: &mut Swarm<network_behaviour::behaviour::MyBehaviour>,line: &String){
    let mut args = line.split(' ');

    match args.next(){
        Some("store:")=>{
            match args.next(){
                Some(String)=>{
                    //UPLOAD FILE
                },
                None=>{
                    //ERROR
                },
            }
        },
        Some("ls")=>{
            match args.next(){ 
                Some("ps") => {
                    for (i, (peer, _)) in swarm.behaviour_mut().gossipsub.all_peers().enumerate(){
                        println!("[{}] {:?}", i+1, peer);
                    }                },
                _=>{}
            }
        }
        _=>{}
    }
}

pub mod network;
