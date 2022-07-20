use crate::protocol::{
    error::ProtoError,
    packets::{Disconnect, Handshake, State},
    PacketReadExtAsync, PacketWriteExtAsync,
};
use std::{error::Error, fmt::Display, net::SocketAddr};
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

    pub async fn disconnect(mut self, reason: impl Into<String>) {
        if !matches!(self.data.next_state, State::Login) {
            return;
        }

        self.stream
            .write_serialize(Disconnect::new(reason))
            .await
            .ok();
        drop(self.stream)
    }

    pub async fn disconnect_err_chain<E: Error>(self, err: E) -> E {
        self.disconnect(err.to_string()).await;
        err
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

impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}=>{} ({:?})",
            self.address,
            self.destination(),
            self.data.next_state
        )
    }
}
