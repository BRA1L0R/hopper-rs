use crate::protocol::{error::ProtoError, packets::Handshake, PacketExtAsync};
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    address: SocketAddr,
    stream: TcpStream,

    handshake: Handshake,
}

impl Client {
    pub fn destination(&self) -> &str {
        &self.handshake.server_address
    }

    pub async fn handshake(
        (mut stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, ProtoError> {
        let handshake = stream
            .read_packet()
            .await?
            .deserialize_owned::<Handshake>()?;

        Ok(Client {
            address,
            stream,
            handshake,
        })
    }
}
