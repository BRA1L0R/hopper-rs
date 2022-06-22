use crate::protocol::PacketWriteExtAsync;

use super::{client::Client, error::ServerError};
use std::net::SocketAddr;
use tokio::{
    io::{copy, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub trait Router: Send + Sync {
    fn route(&self, client: &Client) -> Result<Destination, ServerError>;
}

#[derive(Clone, Copy)]
pub struct Destination(SocketAddr);

impl Destination {
    pub fn new(addr: SocketAddr) -> Self {
        Self(addr)
    }
}

impl Destination {
    pub async fn connect(self, client: Client) -> Result<(), ServerError> {
        let mut server = TcpStream::connect(self.0)
            .await
            .map_err(ServerError::ServerUnreachable)?;

        server.write_serialize(client.handshake_data).await?;

        let (rc, wc) = client.stream.into_split();
        let (rs, ws) = server.into_split();

        // let client_to_server = async {};
        let pipe = |mut input: OwnedReadHalf, mut output: OwnedWriteHalf| async move {
            copy(&mut input, &mut output).await?;
            output.shutdown().await
        };

        tokio::try_join!(pipe(rc, ws), pipe(rs, wc))
            .map_err(ServerError::Disconnected)
            .map(drop)
    }
}
