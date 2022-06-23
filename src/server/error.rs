use thiserror::Error;

use crate::protocol::error::ProtoError;

#[derive(Error, Debug)]
pub enum HopperError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtoError),

    #[error("no server")]
    NoServer,

    #[error("one of the two parties terminated the connection: {0}")]
    Disconnected(std::io::Error),

    #[error("unable to connect to server: {0}")]
    ServerUnreachable(std::io::Error),
}
