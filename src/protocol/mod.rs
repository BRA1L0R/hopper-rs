// mod deserialize;
pub mod data;
pub mod error;
pub mod packets;
pub mod varint;

use std::io::Cursor;

use async_trait::async_trait;

use tokio::io::{AsyncRead, AsyncReadExt};
pub use varint::VarInt;

use self::{
    data::{Deserialize, PacketId},
    error::ProtoError,
    varint::ReadVarIntExtAsync,
};

#[derive(Debug)]
pub struct Packet {
    packet_id: VarInt,
    data: Vec<u8>,
}

impl Packet {
    fn data_cursor(&self) -> Cursor<&[u8]> {
        Cursor::new(&self.data)
    }

    pub fn deserialize_owned<'a, T>(&'a self) -> Result<T, ProtoError>
    where
        T: Deserialize<Cursor<&'a [u8]>> + PacketId,
    {
        (self.packet_id == T::ID)
            .then(|| T::deserialize(&mut self.data_cursor()))
            .ok_or(ProtoError::UnexpectedPacket)?
    }
}

#[async_trait]
pub trait PacketExtAsync
where
    Self: AsyncRead + Unpin,
{
    /// ### Read uncompressed packets
    /// this method only supports the uncompressed unencrypted
    /// format of minecraft packets.
    async fn read_packet(&mut self) -> Result<Packet, ProtoError> {
        let (_, VarInt(packet_len)) = self.read_varint().await?;
        let packet_len = packet_len as usize;

        let (id_size, packet_id) = self.read_varint().await?;

        let mut data: Vec<u8> = Vec::with_capacity(packet_len - id_size);
        unsafe { data.set_len(packet_len) };
        self.read_exact(&mut data).await?;

        Ok(Packet { packet_id, data })
    }
}

impl<T: AsyncRead + Unpin> PacketExtAsync for T {}
