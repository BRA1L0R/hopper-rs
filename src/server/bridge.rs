pub mod forwarding;
mod piping;

use crate::HopperError;

use self::{
    forwarding::{BungeeCord, ForwardStrategy, Passthrough, ProxyProtocol, RealIP},
    piping::{copy_bidirectional, flush_bidirectional},
};

use super::{
    backend::{Backend, Connected},
    client::{IncomingClient, NextState},
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
