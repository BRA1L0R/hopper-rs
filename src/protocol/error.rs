use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("unexpected packet id")]
    UnexpectedPacket,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("invalid varint size")]
    VarInt,

    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),

    #[error("unhandled enum variant")]
    InvalidVariant,
}
