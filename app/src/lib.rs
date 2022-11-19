use libp2p::{Swarm, kad::{Kademlia, store::MemoryStore, record::Key}};
use network::network_behaviour;
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

pub async fn providing_files(kademlia: &mut Kademlia<MemoryStore>) -> Result<(), Box<dyn Error>> {
    let music_dir = "./assets/music".to_string();

    match Path::new(&music_dir).is_dir(){
        true=>{
            let mut files = fs::read_dir(music_dir.clone()).unwrap();

            match files.next().is_none(){
                false=>{
                    for file in files{
                        kademlia.start_providing(Key::new(&file.unwrap().file_name().to_str().expect("File missing").as_bytes())).unwrap();
                    }
                },
                _=>{println!("Your assets directory is empty.")}  
            }
        }
        false=>{
            fs::create_dir(music_dir.clone()).unwrap();
        }
    }
    Ok(())
}

pub mod network;
