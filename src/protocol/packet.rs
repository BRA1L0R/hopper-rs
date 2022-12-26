use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{
    data::{Deserialize, PacketId, Serialize},
    error::ProtoError,
    varint::{ReadVarIntExtAsync, WriteVarIntExt},
    VarInt, SIZE_LIMIT,
};

#[derive(Debug)]
pub struct Packet {
    packet_id: VarInt,
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
        T: Serialize<Vec<u8>> + PacketId,
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

    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Packet, ProtoError> {
        let (_, VarInt(packet_len)) = reader.read_varint().await?;
        let packet_len = packet_len as usize;

        if packet_len > SIZE_LIMIT {
            return Err(ProtoError::SizeLimit);
        }

        let (id_size, packet_id) = reader.read_varint().await?;

        // creates a buffer with capacity and length set to
        // the received packet length
        let mut buf = Vec::with_capacity(packet_len - id_size);

        // Safety: the buffer is filled the next line
        unsafe { buf.set_len(buf.capacity()) };
        reader.read_exact(&mut buf).await?;

        Ok(Packet {
            packet_id,
            data: buf,
        })
    }

    pub async fn write_into<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<usize, ProtoError> {
        // store temporarily the packet id to calculate its length
        let mut packetid_buf = Cursor::new([0; 5]);
        let pid_len = WriteVarIntExt::write_varint(&mut packetid_buf, self.packet_id).unwrap();

        let mut packet_len_buf = Cursor::new([0; 5]);
        let packet_len = VarInt((pid_len + self.data.len()) as i32);
        let plen_length = packet_len_buf.write_varint(packet_len)?;

        // PacketLen + PacketID + Data

        writer
            .write_all(&packet_len_buf.into_inner()[..plen_length])
            .await?;
        writer
            .write_all(&packetid_buf.into_inner()[..pid_len])
            .await?;
        writer.write_all(&self.data).await?;

        Ok(plen_length + pid_len + self.data.len())
    }
}

// #[async_trait]
// pub trait PacketWriteExtAsync
// where
//     Self: AsyncWrite + Unpin,
// {
// async fn write_packet(
//     &mut self,
//     packet: impl AsRef<Packet> + Send,
// ) -> Result<usize, ProtoError> {
//     let packet = packet.as_ref();

//     // store temporarily the packet id to calculate its length
//     let mut packetid_buf = Cursor::new([0; 5]); // TODO: replace with stack buffer
//     let pid_len = WriteVarIntExt::write_varint(&mut packetid_buf, packet.packet_id).unwrap();

//     let mut packet_len_buf = Cursor::new([0; 5]);
//     let packet_len = VarInt((pid_len + packet.data.len()) as i32);
//     let plen_length = packet_len_buf.write_varint(packet_len)?;

//     self.write_all(&packet_len_buf.into_inner()[..plen_length])
//         .await?;
//     self.write_all(&packetid_buf.into_inner()[..pid_len])
//         .await?;
//     self.write_all(&packet.data).await?;

//     Ok(plen_length + pid_len + packet.data.len())
// }

// efficient in-place serialization
pub async fn write_serialize<T, W>(data: T, writer: &mut W) -> Result<usize, ProtoError>
where
    T: Serialize<Vec<u8>> + PacketId,
    W: AsyncWrite + Unpin,
{
    let mut buf = Vec::new();
    VarInt::from(T::ID).serialize(&mut buf).unwrap();
    data.serialize(&mut buf).unwrap();

    let packet_len = VarInt(buf.len() as i32);
    let mut packet_len_buf = Cursor::new([0; 5]);
    let len_size = packet_len_buf.write_varint(packet_len)?;

    writer
        .write_all(&packet_len_buf.into_inner()[..len_size])
        .await?;
    writer.write_all(&buf).await?;

    Ok(len_size + buf.len())
}
// }

// impl<R: AsyncRead + Unpin> PacketReadExtAsync for R {}
// impl<W: AsyncWrite + Unpin> PacketWriteExtAsync for W {}
