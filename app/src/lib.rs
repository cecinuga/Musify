use libp2p::{Swarm, kad::{Kademlia, store::MemoryStore, record::Key}, PeerId, identity::PublicKey, multihash::Multihash, Multiaddr, request_response::ResponseChannel};
use network::{network_behaviour::{self, behaviour::{FileRequest, FileResponse}}, network_settings::network::PATH};
use rodio::{OutputStream, Sink, Decoder};
use std::{fs::{self, File, read_dir}, error::Error, path::Path, io::{Write, BufReader},};
use network::network_behaviour::behaviour::MyBehaviour;

pub fn request_search(swarm: &mut Swarm<MyBehaviour>, item: String) {
    println!("\n[#]I'm looking for {item}...");
    let peer = swarm.behaviour_mut().kademlia.get_providers(Key::new(&item));
} 
pub fn request_download(swarm: &mut Swarm<MyBehaviour>, peer_id: &PeerId, file_name: String) {
    swarm.behaviour_mut().request_response.send_request(
        peer_id, 
        FileRequest(file_name)
    );
}
pub fn response_download(swarm: &mut Swarm<MyBehaviour>,channel: ResponseChannel<FileResponse>,request: FileRequest){
    let mut file_path = PATH.to_string();
    let mut file_name = format!("{}.mp3",request.0);
    file_path.push_str(&file_name);

    let file_to_send = [
        [file_name.as_bytes().to_vec().len() as u8].to_vec(),
        file_name.as_bytes().to_vec(), 
        std::fs::read(file_path).unwrap(),
    ].concat();

    //println!("[#]Received request {:?}", request);
    //println!("[#]Sending file: {:?}", file_path);
    //println!("{:?}",std::fs::read_dir(PATH.to_string()));
    if let Err(e) = swarm.behaviour_mut().request_response.send_response(channel, FileResponse(file_to_send)){
        println!("[!]Connection to peer has to still open.");
    }
}
pub fn save_file(swarm: &mut Swarm<MyBehaviour>, response: FileResponse){
    let len = response.0[0];
    let file_name = std::str::from_utf8(&response.0[1..=len as usize]).unwrap();
    let file_name_path = format!("{}{}",PATH.as_str(),file_name);
    let file_bytes = &response.0[len as usize + 1..];
    println!("[+]File {} Ricevuto!", file_name);

    if let Ok(mut file_void) = std::fs::File::create(file_name_path){
        match file_void.write_all(file_bytes).unwrap(){   _=>{}   }
    }
}

pub async fn handle_command(swarm: &mut Swarm<network_behaviour::behaviour::MyBehaviour>,line: &String){
    let mut args = line.split(' ');

    match args.next(){
        Some("sto:")=>{
            match args.next(){
                Some(String)=>{
                    //UPLOAD FILE
                },
                None=>{
                    //ERROR
                },
            }
        },
        Some("sh:")=>{
            if let Some(name) = args.next(){
                request_search(swarm, name.to_string());
            } else { println!("[-]Usage: 'sh: <file_name>'") }
        }
        Some("dwn:")=>{
            if let Some(peer_id_input) = args.next(){
                if let Ok(peer_id_multiaddr) = format!("/p2p/{}",peer_id_input.to_string()).parse::<Multiaddr>(){
                    if let Some(file_name) = args.next(){
                        let file_name = file_name.to_string();
                        let peer_id = PeerId::try_from_multiaddr(&peer_id_multiaddr).unwrap();
    
                        request_download(swarm, &peer_id, file_name);
                    } else { println!("[-]Usage: 'dwn: <peer_id> <file_name>'") }
                } 
            } else { println!("[-]Usage: 'dwn: <peer_id> <file_name>'") }
        }
        Some("play:")=>{
            if let Some(file_name) = args.next(){
                let (_stream, handler) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&handler).unwrap();
                let file_path = format!("{}{}.mp3",PATH.as_str(),file_name);
                if let Ok(file) = File::open(file_path){
                    sink.append(Decoder::new(BufReader::new(file)).unwrap());
                    sink.sleep_until_end();
                } else{ println!("[-]File not founded."); }          
            } else { println!("[-]Usage: 'play: <peer_id> <file_name>'"); }
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
