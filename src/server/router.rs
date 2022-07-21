use thiserror::Error;

use crate::protocol::packets::Handshake;

use super::bridge::Bridge;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("no server with such hostname has been found")]
    NoServer,

    #[error("unable to connect to server: {0}")]
    Unreachable(std::io::Error),
}

#[async_trait::async_trait]
pub trait Router: Send + Sync {
    // type Server: ConnectedServer;
    async fn route(&self, client: &Handshake) -> Result<Bridge, RouterError>;
}
