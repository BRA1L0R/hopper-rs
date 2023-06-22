use bytes::Buf;
use netherite::{
    encoding::{packetid::PacketId, str::Str, varint::VarInt},
    DeError, Deserialize, Serialize,
};
use serde_json::json;

pub struct JsonChat(String);

impl JsonChat {
    pub fn new(message: &str) -> Self {
        let json = json!({ "text": message });
        Self(serde_json::to_string(&json).unwrap())
    }
}

#[derive(Serialize)]
pub struct Disconnect<'a> {
    chat: &'a str,
}

impl<'a> Disconnect<'a> {
    pub fn from_chat(chat: &'a JsonChat) -> Self {
        Self { chat: &chat.0 }
    }
}

impl PacketId for Disconnect<'_> {
    const ID: i32 = 0x00;
}

// #[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum State {
    Status = 1,
    Login = 2,
}

impl Serialize for State {
    fn serialize(&self, buf: impl bytes::BufMut) {
        (*self as u8).serialize(buf)
    }

    fn size(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl Deserialize for State {
    fn deserialize(buffer: impl Buf) -> Result<Self, netherite::DeError> {
        let VarInt(state) = VarInt::deserialize(buffer)?;

        match state {
            1 => Ok(State::Status),
            2 => Ok(State::Login),
            _ => Err(DeError::InvalidData),
        }
    }
}

#[derive(Deserialize, Debug)]
/// Handshake with fields shallow copied
/// from the original buffer (server_address)
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: Str,
    pub server_port: u16,
    pub next_state: State,
}

impl PacketId for Handshake {
    const ID: i32 = 0x00;
}

/// All-owned for creation
/// of a new handshake
#[derive(Serialize, Debug)]
pub struct NewHandshake {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: State,
}

impl From<Handshake> for NewHandshake {
    fn from(value: Handshake) -> Self {
        NewHandshake {
            protocol_version: value.protocol_version,
            server_address: value.server_address.to_string(),
            server_port: value.server_port,
            next_state: value.next_state,
        }
    }
}

impl PacketId for NewHandshake {
    const ID: i32 = 0x00;
}

#[derive(Debug, Deserialize)]
pub struct LoginStart {
    pub username: Str,
}

impl PacketId for LoginStart {
    const ID: i32 = 0x00;
}
