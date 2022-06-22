// mod deserialize;
pub mod data;
pub mod error;
pub mod packets;
pub mod varint;

use std::io::Cursor;

use async_trait::async_trait;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
pub use varint::VarInt;

use self::{
    data::{Deserialize, PacketId, Serialize},
    error::ProtoError,
    varint::{ReadVarIntExtAsync, WriteVarIntExtAsync},
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

    pub fn serialize<T>(packet: &T) -> Result<Self, ProtoError>
    where
        T: Serialize<Vec<u8>> + PacketId,
    {
        let mut data = Vec::new();
        packet.serialize(&mut data)?;

        Ok(Self {
            packet_id: VarInt::from(T::ID),
            data,
        })
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
pub trait PacketReadExtAsync
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
        unsafe { data.set_len(packet_len - id_size) };
        self.read_exact(&mut data).await?;

        println!("{id_size}");

        Ok(Packet { packet_id, data })
    }

    // async fn write_packet<P: AsRef<Packet>>(&mut self, packet: P) -> Result<(), ProtoError> {
    //     let packet = packet.as_ref();

    // }
}

#[async_trait]
pub trait PacketWriteExtAsync
where
    Self: AsyncWrite + Unpin,
{
    // efficient in-place serialization
    async fn write_serialize<T>(&mut self, data: T) -> Result<usize, ProtoError>
    where
        T: Serialize<Vec<u8>> + PacketId,
    {
        let mut buf = Vec::new();
        VarInt::from(T::ID).serialize(&mut buf).unwrap();
        data.serialize(&mut buf).unwrap();

        // println!("{buf:?}");

        let packet_len = VarInt(buf.len() as i32);
        let len_size = self.write_varint(packet_len).await?;
        self.write_all(&buf).await?;

        Ok(len_size + buf.len())
    }
}

impl<R: AsyncRead + Unpin> PacketReadExtAsync for R {}
impl<W: AsyncWrite + Unpin> PacketWriteExtAsync for W {}
