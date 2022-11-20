pub mod behaviour {
    use libp2p::{ 
        kad::{
            Kademlia, 
            KademliaEvent, 
            record::store::MemoryStore
        },
        core::upgrade::{ProtocolName, read_length_prefixed, write_length_prefixed},
        gossipsub::{Gossipsub, GossipsubEvent},
        NetworkBehaviour, 
        request_response::{
            RequestResponse, 
            RequestResponseEvent, 
            RequestResponseCodec,
        }, 
        mdns::{ Mdns, MdnsEvent },
    };
    use async_trait::async_trait;
    use tokio::io;
    use futures::{
        prelude::*,
        io::{AsyncRead, AsyncWrite}};
    

    #[derive(Debug, Clone)]
    pub struct FileExchangeProtocol();
    #[derive(Clone)]
    pub struct FileExchangeCodec();
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FileRequest(pub String);
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FileResponse(Vec<u8>);

    impl ProtocolName for FileExchangeProtocol {
        fn protocol_name(&self) -> &[u8]{
            "/file-exchange/1".as_bytes()
        }
    }

    #[async_trait]
    impl RequestResponseCodec for FileExchangeCodec{
        type Protocol = FileExchangeProtocol;
        type Request = FileRequest;
        type Response = FileResponse;

        async fn read_request<T>(
            &mut self,
            _: &FileExchangeProtocol,
            io: &mut T,
        ) -> io::Result<Self::Request> 
        where T: AsyncRead + Unpin + Send, {
            let vec = read_length_prefixed(io, 1_000_000).await?;
            
            if vec.is_empty() {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            Ok(FileRequest(String::from_utf8(vec).unwrap()))
        }

        async fn read_response<T>(
            &mut self,
            _: &FileExchangeProtocol,
            io: &mut T,
        ) -> io::Result<Self::Response>
        where T: AsyncRead + Unpin + Send, {
            let vec = read_length_prefixed(io, 500_000_000).await?;
            
            if vec.is_empty(){
                return Err(io::ErrorKind::UnexpectedEof.into());
            }

            Ok(FileResponse(vec))
        }

        async fn write_request<T>(
            &mut self,
            _: &FileExchangeProtocol,
            io: &mut T,
            FileRequest(data): FileRequest,
        ) -> io::Result<()>
        where T: AsyncWrite + Unpin + Send { 
            write_length_prefixed(io, data).await?;
            io.close().await?;

            Ok(())
        }

        async fn write_response<T>(
            &mut self,
            _: &FileExchangeProtocol,
            io: &mut T,
            FileResponse(data): FileResponse,
        ) -> io::Result<()>
        where T: AsyncWrite + Unpin + Send { 
            write_length_prefixed(io, data).await?;
            io.close().await?;

            Ok(())
        }

    }

    #[derive(NetworkBehaviour)]
    #[behaviour(out_event="MyBehaviourEvent")]
    pub struct MyBehaviour{
        pub request_response: RequestResponse<FileExchangeCodec>,
        pub kademlia: Kademlia<MemoryStore>,
        pub gossipsub: Gossipsub,
        pub mdns: Mdns,
    }

    impl MyBehaviour {
        pub fn new(request_response: RequestResponse<FileExchangeCodec>,kademlia:Kademlia<MemoryStore>,gossipsub: Gossipsub ,mdns: Mdns) -> Self{
            Self{ request_response, kademlia, gossipsub, mdns, }
        }
    }

    pub enum MyBehaviourEvent{
        RequestResponse(RequestResponseEvent<FileRequest, FileResponse>),
        Kademlia(KademliaEvent),
        Gossipsub(GossipsubEvent),
        Mdns(MdnsEvent),
    }
    impl From<RequestResponseEvent<FileRequest, FileResponse>> for MyBehaviourEvent{
        fn from(v: RequestResponseEvent<FileRequest, FileResponse>) -> Self{
            Self::RequestResponse(v)
        }
    }
    impl From<KademliaEvent> for MyBehaviourEvent{
        fn from(v: KademliaEvent) -> Self{
            Self::Kademlia(v)
        }
    }
    impl From<GossipsubEvent> for MyBehaviourEvent{
        fn from(v: GossipsubEvent) -> Self{
            Self::Gossipsub(v)
        }
    }
    impl From<MdnsEvent> for MyBehaviourEvent{
        fn from(v: MdnsEvent) -> Self{
            Self::Mdns(v)
        }
    }

}