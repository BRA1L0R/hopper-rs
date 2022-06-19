use std::io::Read;

use super::{
    data::{Deserialize, PacketId},
    error::ProtoError,
    VarInt,
};

#[derive(Debug)]
pub enum State {
    Status,
    Login,
}

impl<R: Read> Deserialize<R> for State {
    fn deserialize(reader: &mut R) -> Result<Self, super::error::ProtoError> {
        let VarInt(next_state) = VarInt::deserialize(reader)?;
        match next_state {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            _ => Err(ProtoError::InvalidVariant),
        }
    }
}

#[derive(Debug)]
pub struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: u16,
    next_state: State,
}

impl PacketId for Handshake {
    const ID: i32 = 0x00;
}

impl<R: Read> Deserialize<R> for Handshake {
    fn deserialize(reader: &mut R) -> Result<Self, super::error::ProtoError> {
        let protocol_version = VarInt::deserialize(reader)?;
        let server_address = String::deserialize(reader)?;
        let server_port = u16::deserialize(reader)?;
        let next_state = State::deserialize(reader)?;

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }
}
