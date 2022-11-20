use libp2p::{Swarm, kad::{Kademlia, store::MemoryStore, record::Key}, PeerId};
use network::network_behaviour::{self, behaviour::FileRequest};
use std::{fs, error::Error, path::Path,};


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
        Some("search:")=>{
            let item = args.next().expect("Missing name").to_string();
            println!("\n[#]I'm looking for {item}...");
            let peer = swarm.behaviour_mut().kademlia.get_providers(Key::new(&item));
        }
        Some("download: ")=>{
            let peer_id = args.next().expect("Peer missing").as_bytes();
            swarm.behaviour_mut().request_response.send_request(&PeerId::from_bytes(peer_id).unwrap(), FileRequest(args.next().expect("Missing filename").to_string()));
        }
        Some("ls")=>{
            match args.next(){ 
                Some("ps") => {
                    for (i, (peer, _)) in swarm.behaviour_mut().gossipsub.all_peers().enumerate(){
                        println!("[{}] {:?}", i+1, peer);
                    }                
                },
                _=>{}
            }
        }
        Some("help") => {
            println!("Commands...");
            println!("help");
            println!("ls ps");
            println!("search: <file_name>");
        }
        _=>{println!("[#]Command not found.");}
    }
}

pub async fn providing_files(kademlia: &mut Kademlia<MemoryStore>) -> Result<(), Box<dyn Error>> {
    let music_dir = "./assets/music".to_string();

    match Path::new(&music_dir).is_dir(){
        true=>{
            if fs::read_dir(music_dir.clone()).unwrap().count()>0{
                    println!("\t");
                    println!("[#]Start providing files in {}", music_dir.clone());
                    for file in fs::read_dir(music_dir.clone()).unwrap(){
                        let file_name = &file
                            .unwrap()
                            .file_name()
                            .to_str()
                            .expect("File missing")
                            .split(".")
                            .next()
                            .expect("Missing name")
                            .to_string();

                        println!("[#]Start providing: {:?}", file_name);
                        kademlia.start_providing(Key::new(file_name)).unwrap();
                    }
            } else { println!("Your assets directory is empty.")} 
        }
        false=>{
            fs::create_dir(music_dir.clone()).unwrap();
        }
    }
    Ok(())
}

pub mod network;
