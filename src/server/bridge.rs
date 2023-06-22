pub mod forwarding;

use crate::{
    protocol::connection::{Codec, Connection, ConnectionError},
    HopperError,
};

use self::forwarding::{BungeeCord, ForwardStrategy, Passthrough, ProxyProtocol, RealIP};

use super::{
    backend::{Backend, Connected},
    client::{IncomingClient, NextState},
};

use futures::SinkExt;
use netherite::packet::RawPacket;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
};

pub struct Bridge {
    // stream: TcpStream,
    // forwarding: ForwardStrategy,
    server: Backend<Connected>,
    client: IncomingClient,
    forwarding: ForwardStrategy,
}

impl Bridge {
    pub fn new(
        server: Backend<Connected>,
        client: IncomingClient,
        forwarding: ForwardStrategy,
    ) -> Self {
        Self {
            server,
            client,
            forwarding,
        }
    }

    /// handshakes an already connected server and
    /// joins two piping futures, bridging the two connections
    /// at Layer 4.
    ///
    /// Returns the number of bytes transferred between
    /// the client and the server. Tuple is (serverbound, clientbound)
    pub async fn bridge(mut self) -> Result<(u64, u64), HopperError> {
        let server = match (&mut self.client.next_state, self.forwarding) {
            (NextState::Login(login), ForwardStrategy::BungeeCord) => {
                // bungeecord forwarding requires username
                // for UUID calculation

                let login_start = login.data()?;
                let primer = BungeeCord::from_username(self.client.address, &login_start.username);

                self.server.prime(primer, self.client.handshake).await?
            }
            // realip works both for login and ping
            (_, ForwardStrategy::RealIP) => {
                let primer = RealIP::new(self.client.address);
                self.server.prime(primer, self.client.handshake).await?
            }
            (_, ForwardStrategy::ProxyProtocol) => {
                let primer = ProxyProtocol::new(self.client.address);
                self.server.prime(primer, self.client.handshake).await?
            }
            // default handler does not forward anything
            _ => {
                self.server
                    .prime(Passthrough, self.client.handshake)
                    .await?
            }
        };

        let client = self.client.connection;
        let mut server = server.into_inner();

        // if the NextState is login the login packet has been read too.
        // Send it to the server as is.
        if let NextState::Login(ref login) = self.client.next_state {
            server.feed_raw_packet(login).await?;
        }

        let (client, server) = flush_bidirectional(client, server).await?;

        // connect the client and the server in an infinite copy loop
        let transferred = copy_bidirectional(server, client).await;
        Ok(transferred)
    }
}

async fn flush_bidirectional(
    client: Connection,
    server: Connection,
) -> Result<(TcpStream, TcpStream), ConnectionError> {
    let mut client = client.into_inner();
    let mut server = server.into_inner();

    client
        .write_buffer_mut()
        .extend_from_slice(server.read_buffer());

    server
        .write_buffer_mut()
        .extend_from_slice(client.read_buffer());

    <Codec as SinkExt<&RawPacket>>::flush(&mut client).await?;
    <Codec as SinkExt<&RawPacket>>::flush(&mut server).await?;

    debug_assert!(server.write_buffer().is_empty());
    debug_assert!(client.write_buffer().is_empty());

    Ok((client.into_inner(), server.into_inner()))
}

/// Uses an external transferred counter so in an event of an error or
/// when the future gets dropped by the select data still gets recorded
async fn pipe(mut input: OwnedReadHalf, mut output: OwnedWriteHalf, transferred: &mut u64) {
    // Accomodate the average MTU of tcp connections
    let mut buffer = [0u8; 2048];

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
