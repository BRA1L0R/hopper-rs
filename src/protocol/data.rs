use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::{error::ProtoError, varint::WriteVarIntExt, VarInt};

pub trait PacketId {
    const ID: i32;
}

pub trait Deserialize<R>: Sized + 'static {
    fn deserialize(reader: &mut R) -> Result<Self, ProtoError>;
}

impl<R: Read> Deserialize<R> for String {
    fn deserialize(reader: &mut R) -> Result<String, ProtoError> {
        let VarInt(size) = VarInt::deserialize(reader)?;
        let size = size as usize;

        let mut buf = Vec::with_capacity(size);

        // Safety: buf is read next line
        unsafe { buf.set_len(size) };
        reader.read_exact(&mut buf)?;

        String::from_utf8(buf).map_err(Into::into)
    }
}

impl<R: Read> Deserialize<R> for u16 {
    fn deserialize(reader: &mut R) -> Result<Self, ProtoError> {
        reader.read_u16::<BigEndian>().map_err(Into::into)
    }
}

pub trait Serialize<W>: Sized + Send {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError>;
}

impl<W: Write> Serialize<W> for &str {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        let len = VarInt(self.len().try_into().unwrap());

        writer.write_varint(len)?;
        writer.write_all(self.as_bytes()).map_err(Into::into)
    }
}

impl<W: Write> Serialize<W> for String {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        self.as_str().serialize(writer)
    }
}

impl<W: Write> Serialize<W> for u16 {
    fn serialize(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_u16::<BigEndian>(*self).map_err(Into::into)
    }
}
