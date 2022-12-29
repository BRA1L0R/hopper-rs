use std::net::SocketAddr;

use super::{bridge::forwarding::ForwardStrategy, IncomingClient};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("no server with such hostname has been found")]
    NoServer,
}

// #[async_trait::async_trait]
#[derive(Debug, Clone, Copy)]
pub struct Destination {
    address: SocketAddr,
    strategy: ForwardStrategy,
}

impl Destination {
    pub fn new(address: SocketAddr, strategy: ForwardStrategy) -> Self {
        Self { address, strategy }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn strategy(&self) -> ForwardStrategy {
        self.strategy
    }
}

pub trait Router: Send + Sync {
    fn route(&self, client: &mut IncomingClient) -> Result<Destination, RouterError>;
}
