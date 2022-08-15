use std::error::Error;

use thiserror::Error;

use crate::protocol::packets::Handshake;

use super::bridge::Bridge;

#[async_trait::async_trait]
pub trait Router: Send + Sync {
    type Error: Error;
    async fn route(&self, client: &Handshake) -> Result<Bridge, Self::Error>;
}
