use std::{
    io::{Read, Write},
    mem::size_of,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::{error::ProtoError, varint::WriteVarIntExt, VarInt};

pub trait PacketId {
    const ID: i32;
}

pub trait Deserialize: Sized + 'static {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, ProtoError>;
}

impl Deserialize for String {
    fn deserialize<R: Read>(reader: &mut R) -> Result<String, ProtoError> {
        const MAX_CHARS: usize = 32767;
        const MAX_BYTES: usize = MAX_CHARS * 4;

        let VarInt(size) = VarInt::deserialize(reader)?;
        let size = size as usize;

        // if exceeds bounds return with error
        if !(0..MAX_BYTES).contains(&size) {
            return Err(ProtoError::Size);
        }

        let mut buf = Vec::with_capacity(size);
        // Safety: buf is read next line
        unsafe { buf.set_len(size) };
        reader.read_exact(&mut buf)?;

        String::from_utf8(buf).map_err(Into::into)
    }
}

impl Deserialize for u16 {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, ProtoError> {
        reader.read_u16::<BigEndian>().map_err(Into::into)
    }
}

pub trait Serialize: Sized + Send {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProtoError>;

    fn min_size(&self) -> usize {
        0
    }
}

impl Serialize for &str {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProtoError> {
        let len = VarInt(self.len().try_into().unwrap());

        writer.write_varint(len)?;
        writer.write_all(self.as_bytes()).map_err(Into::into)
    }

    fn min_size(&self) -> usize {
        self.as_bytes().len() + 3
    }
}

impl Serialize for String {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProtoError> {
        self.as_str().serialize(writer)
    }

    fn min_size(&self) -> usize {
        self.as_str().min_size()
    }
}

impl Serialize for u16 {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_u16::<BigEndian>(*self).map_err(Into::into)
    }

    fn min_size(&self) -> usize {
        size_of::<u16>()
    }
}
