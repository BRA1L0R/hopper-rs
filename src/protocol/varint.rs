use async_trait::async_trait;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{
    io::{Cursor, Read, Write},
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
                return Ok((pos + 1, VarInt(res)));
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

#[async_trait]
pub trait WriteVarIntExtAsync
where
    Self: Unpin + AsyncWrite,
{
    // todo: rewrite in rust
    async fn write_varint(&mut self, varint: VarInt) -> Result<usize, ProtoError> {
        let mut buf = Cursor::new([0; 4]);
        let written = WriteVarIntExt::write_varint(&mut buf, varint).unwrap();

        self.write_all(&buf.into_inner()[..written])
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
        let mut val: u32 = unsafe { mem::transmute(val) };
        let mut written = 0;

        loop {
            written += 1;
            if (val & (!0x7F)) == 0 {
                break self
                    .write_u8(val as u8)
                    .map(|_| written)
                    .map_err(Into::into);
            }

            self.write_u8((val as u8).mask_data().add_continue())?;

            val >>= 7;
        }
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

impl From<i32> for VarInt {
    fn from(val: i32) -> Self {
        VarInt(val)
    }
}
