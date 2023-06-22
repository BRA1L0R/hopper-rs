use std::{fmt::Write, net::SocketAddr};

use bytes::BufMut;

use proxy_protocol::{
    version2::{ProxyAddresses, ProxyCommand, ProxyTransportProtocol},
    ProxyHeader,
};
use serde::Deserialize;

use crate::{
    protocol::{
        connection::Connection,
        packet::DecodedPacket,
        packet_impls::{Handshake, NewHandshake},
        types::PlayerUuid,
    },
    HopperError,
};

#[derive(Debug, Default, Deserialize, Clone, Copy)]
pub enum ForwardStrategy {
    #[default]
    #[serde(rename = "none")]
    None,

    #[serde(rename = "bungeecord")]
    BungeeCord,

    // RealIP <=2.4 support
    #[serde(rename = "realip")]
    RealIP,

    #[serde(rename = "proxy_protocol")]
    ProxyProtocol,
}

#[async_trait::async_trait]
pub trait ConnectionPrimer {
    /// method for priming the connection of a server
    /// which may be with address forwarding informations
    /// or not, up to the implementer
    ///
    /// `og_handshake` is the original handshake that was sent to hoppper
    /// by the client
    async fn prime_connection(
        self,
        stream: &mut Connection,
        og_handshake: DecodedPacket<Handshake>,
    ) -> Result<(), HopperError>;
}

pub(super) struct BungeeCord {
    player_addr: SocketAddr,
    player_uuid: PlayerUuid,
}

impl BungeeCord {
    pub fn from_username(player_addr: SocketAddr, player_name: &str) -> Self {
        Self {
            player_addr,
            // calculate the player's offline UUID. It will get
            // ignored by online-mode servers so we can always send
            // it even when the server is premium-only
            player_uuid: PlayerUuid::offline_player(player_name),
        }
    }
}

#[async_trait::async_trait]
impl ConnectionPrimer for BungeeCord {
    async fn prime_connection(
        self,
        stream: &mut Connection,
        og_handshake: DecodedPacket<Handshake>,
    ) -> Result<(), HopperError> {
        let handshake = og_handshake.into_data();

        // if handshake contains a null character it means that
        // someone is trying to hijack the connection or trying to
        // connect through another proxy
        if handshake.server_address.contains('\x00') {
            return Err(HopperError::Invalid);
        }

        // https://github.com/SpigotMC/BungeeCord/blob/8d494242265790df1dc6d92121d1a37b726ac405/proxy/src/main/java/net/md_5/bungee/ServerConnector.java#L91-L106

        let mut handshake: NewHandshake = handshake.into();
        write!(
            handshake.server_address,
            "\x00{}\x00{}",
            self.player_addr.ip(),
            self.player_uuid
        )
        .ok();

        // send the modified handshake
        // packet::write_serialize(handshake, stream).await?;
        stream.feed_packet(handshake).await?;

        Ok(())
    }
}

pub struct RealIP {
    player_addr: SocketAddr,
}

impl RealIP {
    pub fn new(player_addr: SocketAddr) -> Self {
        Self { player_addr }
    }
}

#[async_trait::async_trait]
impl ConnectionPrimer for RealIP {
    async fn prime_connection(
        self,
        stream: &mut Connection,
        og_handshake: DecodedPacket<Handshake>,
    ) -> Result<(), HopperError> {
        let Handshake {
            protocol_version,
            server_address,
            server_port,
            next_state,
        } = og_handshake.into_data();

        // if the original handshake contains these character
        // the client is trying to hijack realip
        if server_address.contains('/') {
            return Err(HopperError::Invalid);
        }

        // FML support
        let insert_index = server_address
            .find('\x00')
            .map(|a| a - 1)
            .unwrap_or(server_address.len());

        let mut server_address = server_address.to_string();

        // bungeecord and realip forwarding have a very similar structure
        // write!(handshake.server_address, "///{}", client.address).unwrap();
        let realip_data = format!("{}///{}", server_address, self.player_addr);
        server_address.insert_str(insert_index, &realip_data);

        let handshake = NewHandshake {
            protocol_version,
            server_port,
            next_state,
            server_address,
        };

        stream.feed_packet(handshake).await?;

        Ok(())
    }
}

pub struct ProxyProtocol {
    client_addr: SocketAddr,
    dest_addr: SocketAddr,
}

impl ProxyProtocol {
    pub fn new(client_addr: SocketAddr) -> Self {
        let dest_addr = match client_addr {
            SocketAddr::V4(_) => SocketAddr::new([0; 4].into(), 0),
            SocketAddr::V6(_) => SocketAddr::new([0; 16].into(), 0),
        };

        Self {
            client_addr,
            dest_addr,
        }
    }
}

#[async_trait::async_trait]
impl ConnectionPrimer for ProxyProtocol {
    async fn prime_connection(
        self,
        stream: &mut Connection,
        og_handshake: DecodedPacket<Handshake>,
    ) -> Result<(), HopperError> {
        // just send along without doing anything

        // they're either both v4 or both v6
        let proxy_addr = match (self.client_addr, self.dest_addr) {
            (SocketAddr::V4(source), SocketAddr::V4(destination)) => ProxyAddresses::Ipv4 {
                source,
                destination,
            },
            (SocketAddr::V6(source), SocketAddr::V6(destination)) => ProxyAddresses::Ipv6 {
                source,
                destination,
            },
            _ => unreachable!(),
        };

        let header = proxy_protocol::encode(ProxyHeader::Version2 {
            command: ProxyCommand::Proxy,
            transport_protocol: ProxyTransportProtocol::Stream,
            addresses: proxy_addr,
        })
        .unwrap();

        // write proxy header
        stream.write_buffer().put_slice(&header);

        // send along handshake as-is
        // og_handshake.as_ref().write_into(stream).await?;
        stream.feed_raw_packet(og_handshake).await?;

        Ok(())
    }
}

/// Passthrough primer, does not modify the original
/// handshake and just sends along bytes as-is
pub(super) struct Passthrough;

#[async_trait::async_trait]
impl ConnectionPrimer for Passthrough {
    async fn prime_connection(
        self,
        stream: &mut Connection,
        og_handshake: DecodedPacket<Handshake>,
    ) -> Result<(), HopperError> {
        // just send along without doing anything
        stream.feed_raw_packet(og_handshake).await?;
        Ok(())
    }
}
