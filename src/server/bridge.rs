pub mod forwarding;

use crate::HopperError;

use self::forwarding::{BungeeCord, ForwardStrategy, Passthrough, ProxyProtocol, RealIP};

use super::{
    backend::{Backend, Connected},
    client::{IncomingClient, NextState},
};

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

    // pub fn address(&self) -> Result<SocketAddr, HopperError> {
    //     self.stream.peer_addr().map_err(HopperError::Disconnected)
    // }

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
                let username = &login.data()?.username;
                let primer = BungeeCord::from_username(self.client.address, username);

                self.server.prime(primer, self.client.handshake).await?
            }

            // realip works both for login and ping
            (_, ForwardStrategy::RealIP) => {
                let primer = RealIP::new(self.client.address);
                self.server.prime(primer, self.client.handshake).await?
            }

            (_, ForwardStrategy::ProxyProtocol) => {
                let self_addr = self
                    .client
                    .stream
                    .local_addr()
                    .map_err(HopperError::Disconnected)?;
                let primer = ProxyProtocol::new(self.client.address, self_addr)
                    .expect("addresses can't differ");

                self.server.prime(primer, self.client.handshake).await?
            }

            // default handler does not forward anything
            _ => {
                self.server
                    .prime(Passthrough, self.client.handshake)
                    .await?
            }
        };

        let client = self.client.stream;
        let mut server = server.into_inner();

        // if the NextState is login the login packet has been read too.
        // Send it to the server as is.
        if let NextState::Login(login) = self.client.next_state {
            login.as_ref().write_into(&mut server).await?;
        }

        // // connect the client and the server in an infinite copy loop
        let transferred = copy_bidirectional(server, client).await;
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
