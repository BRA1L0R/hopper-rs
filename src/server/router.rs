use crate::{protocol::PacketWriteExtAsync, HopperError};

use super::client::Client;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::{
    io::{copy, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("no server with such hostname has been found")]
    NoServer,

    #[error("unable to connect to server: {0}")]
    Unreachable(std::io::Error),
}

#[async_trait::async_trait]
pub trait Router: Send + Sync {
    async fn route(&self, client: &Client) -> Result<ConnectedServer, RouterError>;
}

#[derive(Debug)]
pub struct ConnectedServer(TcpStream);

impl ConnectedServer {
    pub async fn connect(addr: SocketAddr) -> Result<Self, RouterError> {
        let server = TcpStream::connect(addr)
            .await
            .map_err(RouterError::Unreachable)?;

        Ok(Self(server))
    }

    pub fn endpoint(&self) -> Result<SocketAddr, std::io::Error> {
        self.0.peer_addr()
    }
}

impl ConnectedServer {
    /// handshakes an already connected server and
    /// joins two piping futures, bridging the two connections
    /// at Layer 4.
    ///
    /// Note: hopper does not care what bytes are shared between
    /// the twos
    pub async fn bridge(self, client: Client) -> Result<(), HopperError> {
        let ConnectedServer(mut server) = self;

        server.write_serialize(client.data).await?;

        let (rc, wc) = client.stream.into_split();
        let (rs, ws) = server.into_split();

        // let client_to_server = async {};
        let pipe = |mut input: OwnedReadHalf, mut output: OwnedWriteHalf| async move {
            copy(&mut input, &mut output).await?;
            output.shutdown().await
        };

        tokio::try_join!(pipe(rc, ws), pipe(rs, wc))
            .map_err(HopperError::Disconnected)
            .map(drop)
    }
}
