use thiserror::Error;

use crate::protocol::error::ProtoError;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("protocol error: {0}")]
    Protocol(#[from] ProtoError),
}
