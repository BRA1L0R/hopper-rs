mod read;
mod write;

use std::io::{Read, Write};

use super::{
    data::{Deserialize, Serialize},
    error::ProtoError,
};

pub use read::{ReadVarIntExt, ReadVarIntExtAsync};
pub use write::WriteVarIntExt;

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

impl Deserialize for VarInt {
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, ProtoError> {
        reader.read_varint().map(|(_, varint)| varint)
    }
}

impl Serialize for VarInt {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProtoError> {
        writer.write_varint(*self).map(drop)
    }

    fn min_size(&self) -> usize {
        match self.0 {
            0..=127 => 1,
            _ => 5,
        }
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

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::protocol::varint::ReadVarIntExtAsync;

    use super::VarInt;
    use super::WriteVarIntExt;

    fn test_varint(val: i32, expected: &[u8]) {
        let mut buf = Cursor::new([0; 5]);
        let written = buf.write_varint(VarInt(val)).unwrap();
        assert_eq!(&buf.get_ref()[..written], expected);
        assert_eq!(buf.position(), written as u64);

        let mut expected_reader = Cursor::new(expected);
        let (size, res) = futures::executor::block_on(expected_reader.read_varint()).unwrap();
        assert_eq!(res, val);
        assert_eq!(size, expected.len());
    }

    #[test]
    fn varint_write() {
        test_varint(0, &[0]);
        test_varint(2, &[2]);
        test_varint(255, &[0xFF, 0x01]);
        test_varint(25565, &[0xDD, 0xC7, 0x01]);
        test_varint(-1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }
}
