use crate::{
    protocol::{
        packet::{self},
        uuid::PlayerUuid,
    },
    server::client::NextState,
    HopperError,
};

use super::client::IncomingClient;
use serde::Deserialize;
use std::{fmt::Write, net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
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
    ) -> Result<Self, std::io::Error> {
        // timeout after 5 seconds of trying to connect to server
        let connect = TcpStream::connect(addr);
        let stream = tokio::time::timeout(Duration::from_secs(5), connect).await??;

        Ok(Self { stream, forwarding })
    }

    pub fn address(&self) -> Result<SocketAddr, HopperError> {
        self.stream.peer_addr().map_err(HopperError::Disconnected)
    }

    /// handshakes an already connected server and
    /// joins two piping futures, bridging the two connections
    /// at Layer 4.
    ///
    /// Returns the number of bytes transferred between
    /// the client and the server. Tuple is (serverbound, clientbound)
    pub async fn bridge(self, mut client: IncomingClient) -> Result<(u64, u64), HopperError> {
        let Bridge {
            stream: mut server,
            forwarding,
        } = self;

        match (&mut client.next_state, forwarding) {
            // realip supports ping ip forwarding too, so catching both
            // cases here
            (_, ForwardStrategy::RealIP) => {
                let mut handshake = client.handshake.into_data();

                // if the original handshake contains these character
                // the client is trying to hijack realip
                if handshake.server_address.contains('/') {
                    return Err(HopperError::Invalid);
                }

                // FML support
                let insert_index = handshake
                    .server_address
                    .find('\x00')
                    .map(|a| a - 1)
                    .unwrap_or(handshake.server_address.len());

                // bungeecord and realip forwarding have a very similar structure
                // write!(handshake.server_address, "///{}", client.address).unwrap();
                let realip_data = format!("///{}", client.address);
                handshake
                    .server_address
                    .insert_str(insert_index, &realip_data);

                // server.write_serialize(handshake).await?;
                packet::write_serialize(handshake, &mut server).await?;
            }

            // when next_state is status we don't have a loginstart message
            // to send along so we just send the handshake.
            // ignore any possible forwarding strategy as it does
            // not apply to status pings.
            //
            // also when the forwardstrategy is none we can just send along.
            (NextState::Status, _) | (_, ForwardStrategy::None) => {
                client.handshake.as_ref().write_into(&mut server).await?;
            }

            // requires decoding logindata and reconstructing the handshake packet
            (NextState::Login(login), ForwardStrategy::BungeeCord) => {
                let mut handshake = client.handshake.into_data();
                let logindata = login.data()?;

                // calculate the player's offline UUID. It will get
                // ignored by online-mode servers so we can always send
                // it even when the server is premium-only
                let uuid = PlayerUuid::offline_player(&logindata.username);

                // if handshake contains a null character it means that
                // someone is trying to hijack the connection or trying to
                // connect through another proxy
                if handshake.server_address.contains('\x00') {
                    return Err(HopperError::Invalid);
                }

                // https://github.com/SpigotMC/BungeeCord/blob/8d494242265790df1dc6d92121d1a37b726ac405/proxy/src/main/java/net/md_5/bungee/ServerConnector.java#L91-L106
                write!(
                    handshake.server_address,
                    "\x00{}\x00{}",
                    client.address.ip(),
                    uuid
                )
                .unwrap();

                packet::write_serialize(handshake, &mut server).await?;
            }
        };

        // if the NextState is login the login packet has been read too.
        // Send it to the server as is.
        if let NextState::Login(login) = client.next_state {
            login.as_ref().write_into(&mut server).await?;
        }

        // connect the client and the server in an infinite copy loop
        let transferred = copy_bidirectional(server, client.stream).await;
        Ok(transferred)
    }
}

/// Uses an external transferred counter so in an event of an error or
/// when the future gets dropped by the select data still gets recorded
async fn pipe(mut input: OwnedReadHalf, mut output: OwnedWriteHalf, transferred: &mut u64) {
    // A 1024 bytes buffer should be big enough
    let mut buffer = [0u8; 1024];

    // read from the socket into the buffer, increment the transfer counter
    // and then write all to the other end of the pipe
    loop {
        let size = match input.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(p) => p,
        };

        *transferred += size as u64; // always safe doing

        if output.write_all(&buffer[..size]).await.is_err() {
            break;
        }
    }
}

async fn copy_bidirectional(server: TcpStream, client: TcpStream) -> (u64, u64) {
    let mut serverbound = 0;
    let mut clientbound = 0;

    let (rs, ws) = server.into_split();
    let (rc, wc) = client.into_split();

    // select ensures that when one pipe finishes
    // the other one gets dropped. Fixes socket leak
    // which kept sockets in a WAIT state forever
    select! {
        _ = pipe(rc, ws, &mut serverbound) => {},
        _ = pipe(rs, wc, &mut clientbound) => {}
    };

    (serverbound, clientbound)
}
