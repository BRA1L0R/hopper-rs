use std::io::{Read, Write};

use hopper_macros::Deserialize;

use super::{
    data::{Deserialize, PacketId, Serialize},
    error::ProtoError,
    VarInt,
};

#[derive(Debug, Clone, Copy)]
pub enum State {
    Status = 1,
    Login = 2,
}

impl<W: Write> Serialize<W> for State {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        VarInt(*self as i32).serialize(writer)
    }
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

#[derive(Debug, Deserialize)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: State,
}

impl PacketId for Handshake {
    const ID: i32 = 0x00;
}

impl<W: Write> Serialize<W> for Handshake {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        self.protocol_version.serialize(writer)?;
        self.server_address.serialize(writer)?;
        self.server_port.serialize(writer)?;
        self.next_state.serialize(writer)?;

        Ok(())
    }
}

pub struct Disconnect {
    
}
