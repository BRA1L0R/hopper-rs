use crate::{
    protocol::{
        error::ProtoError,
        lazy::{DecodedPacket, LazyPacket},
        packet::{self, Packet},
        packets::{Disconnect, Handshake, LoginStart, State},
    },
    HopperError,
};
use std::{error::Error, net::SocketAddr, ops::Deref, str::FromStr, sync::Arc, time::Duration};
use tokio::net::TcpStream;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// verified hostname destination
pub struct Destination(Arc<str>);

pub struct NoHostname;

impl Destination {
    pub fn into_inner(self) -> Arc<str> {
        self.0
    }
}

impl FromStr for Destination {
    type Err = NoHostname;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(|c| c == '\x00' || c == '/')
            .next()
            .ok_or(NoHostname)
            .map(Into::into)
            .map(Destination)
    }
}

impl Deref for Destination {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub enum NextState {
    Login(LazyPacket<LoginStart>),
    Status,
}

pub struct IncomingClient {
    pub address: SocketAddr,
    pub stream: TcpStream,

    pub destination: Destination,

    pub handshake: DecodedPacket<Handshake>,
    pub next_state: NextState,
}

impl IncomingClient {
    pub async fn disconnect(mut self, reason: impl Into<String>) {
        if !matches!(self.handshake.data().next_state, State::Login) {
            return;
        }

        packet::write_serialize(Disconnect::new(reason), &mut self.stream)
            .await
            .ok();
    }

    pub async fn disconnect_err(self, err: impl Error) {
        self.disconnect(err.to_string()).await;
    }

    async fn handshake_inner(
        (mut stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, HopperError> {
        let handshake: DecodedPacket<Handshake> =
            Packet::read_from(&mut stream).await?.try_into()?;
        let destination = handshake
            .data()
            .server_address
            .parse()
            .map_err(|_| HopperError::Invalid)?;

        // only read LoginStart information (containing the username)
        // if the next_state is login
        let next_state = match handshake.data().next_state {
            State::Status => NextState::Status,
            State::Login => NextState::Login(Packet::read_from(&mut stream).await?.try_into()?),
        };

        Ok(IncomingClient {
            address,
            stream,
            destination,
            handshake,
            next_state,
        })
    }

    pub async fn handshake(connection: (TcpStream, SocketAddr)) -> Result<Self, HopperError> {
        tokio::time::timeout(Duration::from_secs(2), Self::handshake_inner(connection))
            .await
            .map_err(|_| HopperError::TimeOut)?
    }
}

impl std::fmt::Display for IncomingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}
