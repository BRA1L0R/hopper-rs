use std::io::{Read, Write};

use hopper_macros::{Deserialize, Serialize};
use serde_json::json;

use super::{
    data::{Deserialize, PacketId, Serialize},
    error::ProtoError,
    VarInt,
};

pub struct Chat(String);

impl<W: Write> Serialize<W> for Chat {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        let chat = json!({ "text": self.0 });
        let chat = serde_json::to_string(&chat).unwrap();

        chat.serialize(writer)
    }
}

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: State,
}

impl PacketId for Handshake {
    const ID: i32 = 0x00;
}

#[derive(Serialize)]
pub struct Disconnect {
    chat: Chat,
}

impl Disconnect {
    pub fn new(reason: impl Into<String>) -> Self {
        let chat = Chat(reason.into());
        Self { chat }
    }
}

impl PacketId for Disconnect {
    const ID: i32 = 0x00;
}
