//! Destination server

use std::marker::PhantomData;
use tokio::net::TcpStream;

use super::{bridge::forwarding::ConnectionPrimer, router::Destination};
use crate::{
    protocol::{connection::Connection, packet::DecodedPacket, packet_impls::Handshake},
    HopperError,
};

pub trait BackendState {}

pub struct Connected(());
impl BackendState for Connected {}
pub struct Primed(());
impl BackendState for Primed {}

pub struct Backend<S: BackendState> {
    stream: Connection,
    _state: PhantomData<S>,
}

impl Backend<Connected> {
    pub async fn connect(destination: &Destination) -> Result<Self, HopperError> {
        let stream = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            TcpStream::connect(destination.address()),
        )
        .await
        .map_err(|_| HopperError::TimeOut)?
        .map_err(HopperError::Connect)?;

        let stream = Connection::new(stream);

        Ok(Backend {
            stream,
            _state: Default::default(),
        })
    }

    pub async fn prime(
        mut self,
        primer: impl ConnectionPrimer,
        handshake: DecodedPacket<Handshake>,
    ) -> Result<Backend<Primed>, HopperError> {
        primer.prime_connection(&mut self.stream, handshake).await?;

        Ok(Backend {
            stream: self.stream,
            _state: Default::default(),
        })
    }
}

impl Backend<Primed> {
    pub fn into_inner(self) -> Connection {
        self.stream
    }
}
