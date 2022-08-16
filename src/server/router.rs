use crate::protocol::packets::Handshake;
use std::error::Error;

use super::bridge::Bridge;

#[async_trait::async_trait]
pub trait Router: Send + Sync {
    type Error: Error;
    async fn route(&self, client: &Handshake) -> Result<Bridge, Self::Error>;
}
