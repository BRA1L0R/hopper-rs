use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

use super::{error::ProtoError, VarInt};

pub trait PacketId {
    const ID: i32;
}

pub trait Deserialize<R>: Sized {
    fn deserialize(reader: &mut R) -> Result<Self, ProtoError>;
}

impl<R: Read> Deserialize<R> for String {
    fn deserialize(reader: &mut R) -> Result<String, ProtoError> {
        let VarInt(size) = VarInt::deserialize(reader)?;
        let size = size as usize;

        let mut buf = Vec::with_capacity(size);
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

// impl<R: Buf> Deserialize<R> for &str {
//     fn deserialize(reader: &mut R) -> std::io::Result<Self> {
//         let VarInt(size) = VarInt::deserialize(reader.reader())?;
//         let size = size as usize;

//         // reader.remaining_slice()
//         // reader.inner
//         todo!()
//     }
// }
