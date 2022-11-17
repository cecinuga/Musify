use std::fs::File;

use libp2p::{ 
    core::upgrade::{ProtocolName, read_length_prefixed, write_length_prefixed},
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
pub struct FileRequest(String);
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
    pub mdns: Mdns,
}

impl MyBehaviour {
    pub fn new(request_response: RequestResponse<FileExchangeCodec>, mdns: Mdns) -> Self{
        Self{ request_response, mdns }
    }
}

pub enum MyBehaviourEvent{
    RequestResponse(RequestResponseEvent<FileRequest, FileResponse>),
    Mdns(MdnsEvent),
}
impl From<RequestResponseEvent<FileRequest, FileResponse>> for MyBehaviourEvent{
    fn from(v: RequestResponseEvent<FileRequest, FileResponse>) -> Self{
        Self::RequestResponse(v)
    }
}

impl From<MdnsEvent> for MyBehaviourEvent{
    fn from(v: MdnsEvent) -> Self{
        Self::Mdns(v)
    }
}
