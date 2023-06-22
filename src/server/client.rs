use netherite::encoding::str::Str;
use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    net::SocketAddr,
    ops::Deref,
    time::Duration,
};
use tokio::net::TcpStream;

use crate::{
    protocol::{
        connection::Connection,
        packet::{DecodedPacket, LazyPacket},
        packet_impls::{Disconnect, Handshake, JsonChat, LoginStart, State},
    },
    HopperError,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// verified hostname destination
pub struct Hostname(Str);

impl Hostname {
    fn from_str(s: &Str) -> Option<Self> {
        let substr = s
            .split(|c| c == '\x00' || c == '/')
            .next()
            .filter(|&str| !str.is_empty())?;

        let inner = s.slice(substr);

        Some(Self(inner))
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

// TODO: reorder fields

pub struct IncomingClient {
    /// user source address
    pub address: SocketAddr,
    pub connection: Connection,

    pub handshake: DecodedPacket<Handshake>,
    pub next_state: NextState,

    /// Sanitized hostname, differs from handshake.server_address as that
    /// may still contain extra information.
    pub hostname: Hostname,
}

impl IncomingClient {
    pub async fn disconnect(mut self, reason: impl AsRef<str>) {
        if matches!(self.next_state, NextState::Status) {
            return;
        }

        let chat = JsonChat::new(reason.as_ref());

        self.connection
            .feed_packet(Disconnect::from_chat(&chat))
            .await
            .ok();
        self.connection.flush().await.ok();
    }

    pub async fn disconnect_err(self, err: impl Error) {
        self.disconnect(err.to_string()).await;
    }

    async fn handshake_inner(
        (stream, address): (TcpStream, SocketAddr),
    ) -> Result<Self, HopperError> {
        let mut connection = Connection::new(stream);
        let handshake: DecodedPacket<Handshake> = connection.read_packet().await?.try_into()?;

        let hostname = Hostname::from_str(&handshake.server_address).ok_or(HopperError::Invalid)?;

        // only read LoginStart information (containing the username)
        // if the next_state is login
        let next_state = match handshake.next_state {
            State::Status => NextState::Status,
            State::Login => NextState::Login(connection.read_packet().await?.try_into()?),
        };

        Ok(IncomingClient {
            address,
            connection,
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
}

impl std::fmt::Display for IncomingClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.address.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use netherite::encoding::str::Str;

    use super::Hostname;

    #[test]
    fn test_hostname() {
        let hostname = Str::from_static("hello\x00extra");

        let res = Hostname::from_str(&hostname).unwrap();
        assert_eq!(&res.0, "hello")
    }

    #[test]
    fn test_invalid() {
        let hostname = Str::from_static("\x00extra");
        let res = Hostname::from_str(&hostname);

        assert!(res.is_none())
    }
}
