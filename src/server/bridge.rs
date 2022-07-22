use crate::{
    protocol::{uuid::PlayerUuid, PacketWriteExtAsync},
    server::client::NextState,
    HopperError,
};

use super::{client::IncomingClient, router::RouterError};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::{
    io::{copy, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

#[cfg(feature = "buffered")]
use tokio::io::{BufReader, BufWriter};

#[derive(Debug, Default, Deserialize, Clone, Copy)]
pub enum ForwardStrategy {
    #[default]
    #[serde(rename = "none")]
    None,

    #[serde(rename = "bungeecord")]
    BungeeCord,
}

#[derive(Debug)]
pub struct Bridge {
    stream: TcpStream,
    forwarding: ForwardStrategy,
}

impl Bridge {
    pub async fn connect(
        addr: SocketAddr,
        forwarding: ForwardStrategy,
    ) -> Result<Self, RouterError> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(RouterError::Unreachable)?;

        Ok(Self { stream, forwarding })
    }

    pub fn address(&self) -> Result<SocketAddr, HopperError> {
        self.stream.peer_addr().map_err(HopperError::Disconnected)
    }

    /// handshakes an already connected server and
    /// joins two piping futures, bridging the two connections
    /// at Layer 4.
    ///
    /// Note: hopper does not care what bytes are shared between
    /// the twos
    pub async fn bridge(self, client: IncomingClient) -> Result<(), HopperError> {
        let Bridge {
            mut stream,
            forwarding,
        } = self;

        match (client.next_state, forwarding) {
            // when next_state is status we don't have a loginstart message
            // to send along so we just send the handshake.
            //
            // ignore any possible forwarding strategy as it does
            // not apply to status pings
            (NextState::Status, _) => stream.write_packet(client.handshake).await?,

            // reuse the same packet data that came in-bound (without even decoding
            // the login start packet!) ensuring max efficiency
            (NextState::Login(login), ForwardStrategy::None) => {
                stream.write_packet(client.handshake).await?;
                stream.write_packet(login).await?
            }

            // requires decoding logindata and reconstructing the handshake packet
            (NextState::Login(mut login), ForwardStrategy::BungeeCord) => {
                // decode handshake
                let mut handshake = client.handshake.into_data()?;
                // decode info from LoginStart
                let logindata = login.data()?;

                // calculate the player's offline UUID. It will get
                // ignored by online-mode servers so we can always send
                // it even when the server is premium-only
                let uuid = PlayerUuid::offline_player(&logindata.username);

                // https://github.com/SpigotMC/BungeeCord/blob/8d494242265790df1dc6d92121d1a37b726ac405/proxy/src/main/java/net/md_5/bungee/ServerConnector.java#L91-L106
                handshake.server_address = format!(
                    "{}\x00{}\x00{}",
                    handshake.server_address,
                    client.address.ip(),
                    uuid
                );

                println!("{handshake:?} {logindata:?} {uuid}");

                stream.write_serialize(handshake).await?;
                stream.write_packet(login).await?
            }
        };

        let (rc, wc) = client.stream.into_split();
        let (rs, ws) = stream.into_split();

        #[cfg(feature = "buffered")]
        let pipe = |mut input: BufReader<OwnedReadHalf>, mut output: BufWriter<OwnedWriteHalf>| async move {
            copy(&mut input, &mut output).await?;
            output.shutdown().await
        };

        #[cfg(not(feature = "buffered"))]
        let pipe = |mut input: OwnedReadHalf, mut output: OwnedWriteHalf| async move {
            copy(&mut input, &mut output).await?;
            output.shutdown().await
        };

        // create two futures, one that copies server->client and the other client->server
        // then join them together to make them work on the same task concurrently
        #[cfg(feature = "buffered")]
        tokio::try_join!(
            pipe(BufReader::new(rc), BufWriter::new(ws)),
            pipe(BufReader::new(rs), BufWriter::new(wc))
        )
        .map_err(HopperError::Disconnected)?;

        #[cfg(not(feature = "buffered"))]
        tokio::try_join!(pipe(rc, ws), pipe(rs, wc)).map_err(HopperError::Disconnected)?;

        Ok(())
    }
}
