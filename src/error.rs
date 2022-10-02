use thiserror::Error;

use crate::{
    config::ServerConfigError,
    // metrics::MetricsError,
    protocol::error::ProtoError,
    server::router::RouterError,
};

#[derive(Error, Debug)]
pub enum HopperError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtoError),

    #[error("routing error: {0}")]
    Router(#[from] RouterError),

    #[error("one of the two parties terminated the connection: {0}")]
    Disconnected(std::io::Error),

    #[error("one of the two parties took too long to respond")]
    TimeOut,

    #[error("the user sent invalid handshake data")]
    Invalid,

    #[error("configuration error: {0}")]
    Config(#[from] ServerConfigError),

    #[error("cannot listen on the specified ip: {0}")]
    Bind(std::io::Error),
    
    #[error("received sigint signal")]
    Signal,
}
