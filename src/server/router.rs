use crate::protocol::PacketWriteExtAsync;

use super::{client::Client, error::HopperError};
use std::net::SocketAddr;
use tokio::{
    io::{copy, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub trait Router: Send + Sync {
    fn route(&self, client: &Client) -> Result<SocketAddr, HopperError>;
}

#[derive(Debug)]
pub struct Server(TcpStream);

impl Server {
    pub async fn connect(addr: SocketAddr) -> Result<Self, HopperError> {
        let server = TcpStream::connect(addr)
            .await
            .map_err(HopperError::ServerUnreachable)?;

        Ok(Self(server))
    }
}

impl Server {
    pub async fn bridge(self, client: Client) -> Result<(), HopperError> {
        let Server(mut server) = self;

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
