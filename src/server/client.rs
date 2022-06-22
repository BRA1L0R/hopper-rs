use crate::protocol::{error::ProtoError, packets::Handshake, PacketReadExtAsync};
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    pub address: SocketAddr,
    pub stream: TcpStream,

    pub handshake_data: Handshake,
}

impl Client {
    pub fn destination(&self) -> &str {
        &self.handshake_data.server_address
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
            handshake_data: handshake,
        })
    }
}
