use super::{
    data::{Deserialize, PacketId},
    error::ProtoError,
    Packet,
};
use std::io::Cursor;

/// Represents a decoded packet. While lazy packet may be
/// decoded or still be raw, this struct sgnifies that a decoding
/// action took place and doesn't require a &mut self for accessing
/// the data inside
pub struct DecodedPacket<T: PacketId> {
    packet: Packet,
    data: T,
}

impl<T: PacketId> DecodedPacket<T> {
    pub fn into_data(self) -> T {
        self.data
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

impl<T: PacketId + for<'a> Deserialize<Cursor<&'a [u8]>>> TryFrom<Packet> for DecodedPacket<T> {
    type Error = ProtoError;

    fn try_from(packet: Packet) -> Result<Self, Self::Error> {
        let data = packet.deserialize_owned()?;

        packet
            .is::<T>()
            .then_some(DecodedPacket { packet, data })
            .ok_or(ProtoError::UnexpectedPacket)
    }
}

impl<T: PacketId> AsRef<Packet> for DecodedPacket<T> {
    fn as_ref(&self) -> &Packet {
        &self.packet
    }
}

/// A packet that might have been decoded or not
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

    pub fn decode(self) -> Result<DecodedPacket<T>, ProtoError> {
        let data = match self.data {
            Some(data) => data,
            None => T::deserialize(&mut Cursor::new(&self.packet.data))?,
        };

        Ok(DecodedPacket {
            data,
            packet: self.packet,
        })
    }
}
