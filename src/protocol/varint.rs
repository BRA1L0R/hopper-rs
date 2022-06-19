use async_trait::async_trait;
use byteorder::ReadBytesExt;
use std::io::Read;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{data::Deserialize, error::ProtoError};

pub trait VarIntOp {
    fn is_stop(&self) -> bool;
    fn mask_data(&self) -> u8;
}

impl VarIntOp for u8 {
    #[inline]
    fn is_stop(&self) -> bool {
        self & 0x80 == 0
    }

    #[inline]
    fn mask_data(&self) -> u8 {
        self & 0x7F
    }
}

/// bincode varints are different
/// from minecraft varints
#[derive(Debug)]
pub struct VarInt(pub i32);

impl PartialEq<i32> for VarInt {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

#[async_trait]
pub trait ReadVarIntExtAsync
where
    Self: Unpin + AsyncRead,
{
    async fn read_varint(&mut self) -> Result<(usize, VarInt), ProtoError> {
        let mut res = 0;

        for pos in 0..4 {
            let current_byte = self.read_u8().await?;
            res |= ((current_byte.mask_data()) as i32) << (pos * 7);

            if current_byte.is_stop() {
                return Ok((pos, VarInt(res)));
            }
        }

        Err(ProtoError::VarInt)
    }
}

pub trait ReadVarIntExt
where
    Self: Read,
{
    fn read_varint(&mut self) -> Result<(usize, VarInt), ProtoError> {
        let mut res = 0;

        for pos in 0..4 {
            let current_byte = self.read_u8()?;
            res |= ((current_byte.mask_data()) as i32) << (pos * 7);

            if current_byte.is_stop() {
                return Ok((pos, VarInt(res)));
            }
        }

        Err(ProtoError::VarInt)
    }
}

impl<T: AsyncRead + Unpin> ReadVarIntExtAsync for T {}
impl<T: Read> ReadVarIntExt for T {}

impl<R: Read> Deserialize<R> for VarInt {
    fn deserialize(reader: &mut R) -> Result<Self, ProtoError> {
        reader.read_varint().map(|(_, varint)| varint)
    }
}

// struct SeqReader<'a, A: SeqAccess<'a>>(&'a A);
// impl<'a, A: SeqAccess<'a>> Read for SeqReader<'a, A> {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         todo!()
//     }
// }

// impl<'de> Deserialize<'de> for VarInt {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         pub struct VarIntVisitor;
//         impl<'v> Visitor<'v> for VarIntVisitor {
//             type Value = VarInt;

//             fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 write!(f, "properly sized varint")
//             }

//             fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//             where
//                 A: serde::de::SeqAccess<'v>,
//             {
//                 SeqReader::<'v>(&seq)
//                     .read_varint()
//                     .map_err(Error::custom)
//                     .map(|(_, varint)| varint)
//             }
//         }

//         deserializer.deserialize_seq(VarIntVisitor)
//     }
// }
