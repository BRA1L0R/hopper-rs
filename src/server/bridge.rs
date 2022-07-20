use crate::{protocol::PacketWriteExtAsync, HopperError};

use super::{client::Client, router::RouterError};
use std::net::SocketAddr;
use tokio::{
    io::{copy, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

#[derive(Debug)]
pub struct Bridge(TcpStream);

impl Bridge {
    pub async fn connect(addr: SocketAddr) -> Result<Self, RouterError> {
        let server = TcpStream::connect(addr)
            .await
            .map_err(RouterError::Unreachable)?;

        Ok(Self(server))
    }

    pub fn address(&self) -> Result<SocketAddr, HopperError> {
        self.0.peer_addr().map_err(HopperError::Disconnected)
    }

    /// handshakes an already connected server and
    /// joins two piping futures, bridging the two connections
    /// at Layer 4.
    ///
    /// Note: hopper does not care what bytes are shared between
    /// the twos
    pub async fn bridge(self, client: Client) -> Result<(), HopperError> {
        let Bridge(mut server) = self;

        // send handshake to server
        server.write_serialize(client.data).await?;

        let (rc, wc) = client.stream.into_split();
        let (rs, ws) = server.into_split();

        let pipe = |mut input: OwnedReadHalf, mut output: OwnedWriteHalf| async move {
            copy(&mut input, &mut output).await?;
            output.shutdown().await
        };

        // create two futures, one that copies server->client and the other client->server
        // then join them together to make them work on the same task concurrently
        tokio::try_join!(pipe(rc, ws), pipe(rs, wc))
            .map_err(HopperError::Disconnected)
            // match the function return signature
            .map(drop)
    }
}
