use super::{
    data::{Deserialize, PacketId},
    error::ProtoError,
    Packet,
};
use std::io::Cursor;

pub struct LazyPacket<T: PacketId> {
    packet: Packet,
    data: Option<T>,
}

impl<T: PacketId> TryFrom<Packet> for LazyPacket<T> {
    type Error = ProtoError;

    fn try_from(packet: Packet) -> Result<Self, Self::Error> {
        packet
            .is::<T>()
            .then_some(LazyPacket { packet, data: None })
            .ok_or(ProtoError::UnexpectedPacket)
    }
}

impl<T: PacketId> From<LazyPacket<T>> for Packet {
    fn from(lazy_packet: LazyPacket<T>) -> Self {
        lazy_packet.packet
    }
}

impl<T: PacketId> AsRef<Packet> for LazyPacket<T> {
    fn as_ref(&self) -> &Packet {
        &self.packet
    }
}

impl<T> LazyPacket<T>
where
    T: PacketId + for<'a> Deserialize<Cursor<&'a [u8]>>,
{
    pub fn data(&mut self) -> Result<&T, ProtoError> {
        match self.data {
            Some(ref data) => Ok(data),
            None => {
                let data = self.packet.deserialize_owned::<T>()?;
                Ok(self.data.insert(data))
            }
        }
    }

    pub fn into_data(self) -> Result<T, ProtoError> {
        match self.data {
            Some(data) => Ok(data),
            None => T::deserialize(&mut Cursor::new(&self.packet.data)),
        }
    }
}
