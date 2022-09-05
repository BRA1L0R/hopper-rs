use super::{bridge::Bridge, IncomingClient};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("no server with such hostname has been found")]
    NoServer,

    #[error("unable to connect to server: {0}")]
    Unreachable(std::io::Error),
}

#[async_trait::async_trait]
pub trait Router: Send + Sync {
    async fn route(&self, client: &mut IncomingClient) -> Result<Bridge, RouterError>;
}
