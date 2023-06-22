use crate::protocol::varint::WriteVarIntExt;

use futures::SinkExt;
use netherite::codec::MinecraftCodec;
use netherite::RawPacket;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use super::packet::Packet;
use super::varint::ReadVarIntExtAsync;
use super::SIZE_LIMIT;
use super::{
    encoding::{PacketId, Serialize},
    error::ProtoError,
    VarInt,
};

// type Codec = Framed<TcpStream, MinecraftCodec>;

// pub struct Connection {
//     inner: Codec,
// }

// impl Connection {
//     pub fn new(inner: TcpStream) -> Self {
//         Self {
//             inner: Codec::new(inner, MinecraftCodec::default()),
//         }
//     }

//     pub fn into_inner(self) -> Codec {
//         self.inner
//     }

//     pub fn inner(&self) -> &Codec {
//         &self.inner
//     }

//     pub fn inner_mut(&mut self) -> &mut Codec {
//         &mut self.inner
//     }

// /// Creates a packet with T's packetid and serializes T in it,
// /// then sends it
// pub async fn write_serialize<T>(&mut self, data: T) -> Result<usize, ProtoError>
// where
//     T: Serialize + PacketId,
// {
//     let packet = Packet::serialize(&data)?;
//     self.write_packet(&packet).await
// }

// /// Reads a packet from the stream
// pub async fn read_packet(&mut self) -> Result<Packet, ProtoError> {
//     let (_, VarInt(packet_len)) = self.inner.read_varint().await?;
//     let packet_len = packet_len as usize;

//     // if exceeds bounds then return with error
//     if !(1..SIZE_LIMIT).contains(&packet_len) {
//         return Err(ProtoError::Size);
//     }

//     let (id_size, packet_id) = self.inner.read_varint().await?;

//     // creates a buffer with capacity and length set to
//     // the received packet length
//     let mut buf = Vec::with_capacity(packet_len - id_size);

//     // Safety: the buffer is filled the next line
//     unsafe { buf.set_len(buf.capacity()) };
//     self.inner.read_exact(&mut buf).await?;

//     Ok(Packet {
//         packet_id,
//         data: buf,
//     })
// }

// /// Writes an already serialized packet in the stream
// pub async fn write_packet(&mut self, packet: &Packet) -> Result<usize, ProtoError> {
//     // store temporarily the packet id to calculate its length
//     let mut packetid_buf = Cursor::new([0; 5]);
//     let pid_len = WriteVarIntExt::write_varint(&mut packetid_buf, packet.packet_id).unwrap();

//     let mut packet_len_buf = Cursor::new([0; 5]);
//     let packet_len = VarInt((pid_len + packet.data.len()) as i32);
//     // plen_length = the length of the "packet length" section
//     let plen_length = packet_len_buf.write_varint(packet_len)?;

//     // PacketLen + PacketID + Data
//     self.inner
//         .write_all(&packet_len_buf.into_inner()[..plen_length])
//         .await?;
//     self.inner
//         .write_all(&packetid_buf.into_inner()[..pid_len])
//         .await?;
//     self.inner.write_all(&packet.data).await?;

//     Ok(plen_length + pid_len + packet.data.len())
// }
// }
