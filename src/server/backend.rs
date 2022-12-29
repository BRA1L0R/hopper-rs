//! Destination server

use std::marker::PhantomData;

use tokio::net::TcpStream;

use crate::{
    protocol::{lazy::DecodedPacket, packets::Handshake},
    HopperError,
};

use super::{bridge::forwarding::ConnectionPrimer, router::Destination};

pub trait BackendState {}

pub struct Connected(());
impl BackendState for Connected {}
pub struct Primed(());
impl BackendState for Primed {}

pub struct Backend<S: BackendState> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

impl Backend<Connected> {
    pub async fn connect(destination: &Destination) -> Result<Self, HopperError> {
        let stream = TcpStream::connect(destination.address())
            .await
            .map_err(HopperError::Connect)?;

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
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}
