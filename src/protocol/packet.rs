use std::ops::Deref;

use netherite::{encoding::packetid::PacketId, packet::RawPacket, Deserialize};

use super::connection::ProtoError;

pub struct DecodedPacket<T> {
    packet: RawPacket,
    data: T,
}

impl<T> AsRef<RawPacket> for DecodedPacket<T> {
    fn as_ref(&self) -> &RawPacket {
        &self.packet
    }
}

impl<T> From<DecodedPacket<T>> for RawPacket {
    fn from(DecodedPacket { packet, .. }: DecodedPacket<T>) -> Self {
        packet
    }
}

impl<T: PacketId> DecodedPacket<T> {
    /// gets owned data and drops original packet
    pub fn into_data(self) -> T {
        self.data
    }
}

impl<T> Deref for DecodedPacket<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Deserialize + PacketId> TryFrom<RawPacket> for DecodedPacket<T> {
    type Error = ProtoError;

    fn try_from(packet: RawPacket) -> Result<Self, Self::Error> {
        let data = packet.deserialize().ok_or(ProtoError::Id)??;
        Ok(Self { data, packet })
    }
}

pub struct LazyPacket<T> {
    packet: RawPacket,
    data: Option<T>,
}

impl<T> AsRef<RawPacket> for LazyPacket<T> {
    fn as_ref(&self) -> &RawPacket {
        &self.packet
    }
}

impl<T: Deserialize + PacketId> LazyPacket<T> {
    pub fn data(&mut self) -> Result<&T, ProtoError> {
        match self.data {
            Some(ref data) => Ok(data),
            None => {
                // already checked in TryFrom
                let data = self.packet.deserialize_unchecked()?;
                Ok(self.data.insert(data))
            }
        }
    }
}

impl<T> From<LazyPacket<T>> for RawPacket {
    fn from(LazyPacket { packet, .. }: LazyPacket<T>) -> Self {
        packet
    }
}

impl<T: PacketId> TryFrom<RawPacket> for LazyPacket<T> {
    type Error = ProtoError;

    fn try_from(packet: RawPacket) -> Result<Self, Self::Error> {
        packet
            .is::<T>()
            .then_some(LazyPacket { packet, data: None })
            .ok_or(ProtoError::Id)
    }
}
