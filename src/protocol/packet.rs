use std::io::Cursor;

use super::{
    data::{Deserialize, PacketId, Serialize},
    error::ProtoError,
    VarInt,
};

#[derive(Debug)]
pub struct Packet {
    pub(super) packet_id: VarInt,
    pub(super) data: Vec<u8>,
}

impl Packet {
    fn data_cursor(&self) -> Cursor<&[u8]> {
        Cursor::new(&self.data)
    }

    pub fn is<T: PacketId>(&self) -> bool {
        self.packet_id == T::ID
    }

    pub fn serialize<T>(packet: &T) -> Result<Self, ProtoError>
    where
        T: Serialize + PacketId,
    {
        let mut data = Vec::new();
        packet.serialize(&mut data)?;

        Ok(Self {
            packet_id: VarInt::from(T::ID),
            data,
        })
    }

    pub fn deserialize_owned<T>(&self) -> Result<T, ProtoError>
    where
        T: for<'a> Deserialize<Cursor<&'a [u8]>> + PacketId + 'static,
    {
        (self.packet_id == T::ID)
            .then(|| T::deserialize(&mut self.data_cursor()))
            .ok_or(ProtoError::UnexpectedPacket)?
    }
}
