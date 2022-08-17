use config::ConfigError;
use thiserror::Error;

use crate::{metrics::MetricsError, protocol::error::ProtoError, server::router::RouterError};

#[derive(Error, Debug)]
pub enum HopperError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtoError),

    #[error("routing error: {0}")]
    Router(#[from] RouterError),

    #[error("metrics engine error: {0}")]
    Metrics(#[from] MetricsError),

    #[error("one of the two parties terminated the connection: {0}")]
    Disconnected(std::io::Error),

    #[error("the user sent invalid handshake data")]
    Invalid,

    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("cannot listen on the specified ip: {0}")]
    Bind(std::io::Error),
}
