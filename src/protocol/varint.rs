use async_trait::async_trait;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Write},
    mem,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::{
    data::{Deserialize, Serialize},
    error::ProtoError,
};

pub trait VarIntOp {
    fn has_stop(self) -> bool;
    fn mask_data(self) -> u8;
    fn add_continue(self) -> u8;
}

impl VarIntOp for u8 {
    #[inline]
    fn has_stop(self) -> bool {
        self & 0x80 == 0
    }

    #[inline]
    fn mask_data(self) -> u8 {
        self & 0x7F
    }

    #[inline]
    fn add_continue(self) -> u8 {
        self | 0x80
    }
}

/// bincode varints are different
/// from minecraft varints
#[derive(Debug, Clone, Copy)]
pub struct VarInt(pub i32);

impl PartialEq<i32> for VarInt {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

macro_rules! varintread {
    ($read:expr) => {{
        let mut res = 0;

        for pos in 0..4 {
            let current_byte = $read?;
            res |= ((current_byte.mask_data()) as i32) << (pos * 7);

            if current_byte.has_stop() {
                return Ok((pos, VarInt(res)));
            }
        }

        Err(ProtoError::VarInt)
    }};
}

macro_rules! varintwrite {
    ($buf:ident, $val:ident) => {{
        let mut written = 0;

        loop {
            written += 1;
            if ($val & (!0x7F)) == 0 {
                WriteBytesExt::write_u8(&mut $buf, $val as u8).ok();
                break;
            }

            WriteBytesExt::write_u8(&mut $buf, ($val as u8).mask_data().add_continue()).ok();

            $val >>= 7;
        }

        written
    }};
}

#[async_trait]
pub trait ReadVarIntExtAsync
where
    Self: Unpin + AsyncRead,
{
    async fn read_varint(&mut self) -> Result<(usize, VarInt), ProtoError> {
        varintread!(self.read_u8().await)
    }
}

pub trait ReadVarIntExt
where
    Self: Read,
{
    fn read_varint(&mut self) -> Result<(usize, VarInt), ProtoError> {
        varintread!(self.read_u8())
    }
}

struct VarIntIter(u32);

impl VarIntIter {
    pub fn new(VarInt(val): VarInt) -> Self {
        VarIntIter(unsafe { mem::transmute(val) })
    }
}

impl Iterator for VarIntIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0 => None,
            val @ 0..=0x7F => Some(val as u8),
            val @ 0x80.. => {
                self.0 >>= 7;
                Some((val as u8).mask_data() | 0x80)
            }
        }
    }
}

#[async_trait]
pub trait WriteVarIntExtAsync
where
    Self: Unpin + AsyncWrite,
{
    // todo: rewrite in rust
    async fn write_varint(&mut self, VarInt(val): VarInt) -> Result<usize, ProtoError> {
        let mut buf = Vec::with_capacity(4);
        let mut val: u32 = unsafe { mem::transmute(val) };

        let written = varintwrite!(buf, val);

        self.write_all(&buf)
            .await
            .map(|_| written)
            .map_err(Into::into)
    }
}

pub trait WriteVarIntExt
where
    Self: Write,
{
    fn write_varint(&mut self, VarInt(val): VarInt) -> Result<usize, ProtoError> {
        let mut buf = Vec::with_capacity(4);
        let mut val: u32 = unsafe { mem::transmute(val) };

        let written = varintwrite!(buf, val);

        self.write_all(&buf).map(|_| written).map_err(Into::into)
    }
}

impl<T: AsyncRead + Unpin> ReadVarIntExtAsync for T {}
impl<T: Read> ReadVarIntExt for T {}
impl<T: AsyncWrite + Unpin> WriteVarIntExtAsync for T {}
impl<T: Write> WriteVarIntExt for T {}

impl<R: Read> Deserialize<R> for VarInt {
    fn deserialize(reader: &mut R) -> Result<Self, ProtoError> {
        reader.read_varint().map(|(_, varint)| varint)
    }
}

impl<W: Write> Serialize<W> for VarInt {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_varint(*self).map(drop)
    }
}

impl From<VarInt> for i32 {
    fn from(varint: VarInt) -> Self {
        varint.0
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
