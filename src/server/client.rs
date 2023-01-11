use crate::{
    protocol::{
        connection::Connection,
        lazy::{DecodedPacket, LazyPacket},
        packets::{Disconnect, Handshake, LoginStart, State},
    },
    HopperError,
};
use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    net::SocketAddr,
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::net::TcpStream;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// verified hostname destination
pub struct Hostname(Arc<str>);

pub struct NoHostname;

impl Hostname {
    pub fn into_inner(self) -> Arc<str> {
        self.0
    }
}

impl FromStr for Hostname {
    type Err = NoHostname;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(|c| c == '\x00' || c == '/')
            .next()
            .ok_or(NoHostname)
            .map(Into::into)
            .map(Hostname)
    }
}

impl Deref for Hostname {
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
    /// user source address
    pub address: SocketAddr,
    pub stream: Connection,

    pub handshake: DecodedPacket<Handshake>,

    /// Sanitized hostname, differs from handshake.server_address as that
    /// may still contain extra information.
    pub hostname: Hostname,
    pub next_state: NextState,
}

impl IncomingClient {
    pub async fn disconnect(mut self, reason: impl Into<String>) {
        if matches!(self.handshake.data().next_state, State::Status) {
            return;
        }

        self.stream
            .write_serialize(Disconnect::new(reason))
            .await
            .ok();
    }

    pub async fn disconnect_err(self, err: impl Error) {
        self.disconnect(err.to_string()).await;
    }

    async fn handshake_inner(
        (stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, HopperError> {
        let mut stream = Connection::new(stream);

        let handshake: DecodedPacket<Handshake> = stream.read_packet().await?.try_into()?;

        // sanitize and parse handshake server_address
        let hostname = handshake
            .data()
            .server_address
            .parse()
            .map_err(|_| HopperError::Invalid)?;

        // only read LoginStart information (containing the username)
        // if the next_state is login
        let next_state = match handshake.data().next_state {
            State::Status => NextState::Status,
            State::Login => NextState::Login(stream.read_packet().await?.try_into()?),
        };

        Ok(IncomingClient {
            address,
            stream,
            hostname,
            handshake,
            next_state,
        })
    }

    pub async fn init(connection: (TcpStream, SocketAddr)) -> Result<Self, HopperError> {
        tokio::time::timeout(Duration::from_secs(2), Self::handshake_inner(connection))
            .await
            .map_err(|_| HopperError::TimeOut)?
    }

    /// get a non-cryptographical hash that can be used
    /// in a load balancing operation
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.address.hash(&mut hasher);
        self.hostname.hash(&mut hasher);

        hasher.finish()
    }

    // pub fn connected_to(&self) -> SocketAddr {
    //     self.stream.inner().loca
    //     todo!()
    // }
}

impl std::fmt::Display for IncomingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.address.fmt(f)
    }
}
