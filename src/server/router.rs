use super::{client::Client, error::ServerError};
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub trait Router: Send + Sync {
    fn route(&self, client: &Client) -> Result<Destination, ServerError>;
}

#[derive(Clone, Copy)]
pub struct Destination(SocketAddr);

impl Destination {
    pub async fn connect(self, client: Client) -> Result<(), ServerError> {
        let stream = TcpStream::connect(self.0)
            .await
            .map_err(ServerError::ServerUnreachable)?;

        todo!()
    }
}
