use crate::protocol::{
    error::ProtoError, packets::Handshake, PacketReadExtAsync, PacketWriteExtAsync,
};
use std::{marker::PhantomData, net::SocketAddr};
use tokio::net::TcpStream;

pub struct Client {
    pub address: SocketAddr,
    pub stream: TcpStream,
    pub data: Handshake,
}

impl Client {
    pub fn destination(&self) -> &str {
        &self.data.server_address
    }

    pub fn disconnect(self) {
        // match self.data.next_state {}
        todo!()
    }

    pub async fn handshake(
        (mut stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, ProtoError> {
        let data = stream
            .read_packet()
            .await?
            .deserialize_owned::<Handshake>()?;

        Ok(Client {
            address,
            stream,
            data,
        })
    }
}
