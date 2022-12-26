use byteorder::ReadBytesExt;
use futures::Future;
use std::{
    io::{ErrorKind, Read, Write},
    mem::MaybeUninit,
    pin::Pin,
    task::Poll,
};
use tokio::io::{AsyncRead, ReadBuf};

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

        for pos in 0..5 {
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
        std::iter::successors(Some($val), |val| Some(val >> 7))
            .take_while(|val| *val != 0)
            .map(|val| (val as u8).add_continue())
            .enumerate()
            .for_each(|(pos, val)| {
                written = pos;
                $buf[pos] = val
            });

        // remove continue bit from the last element
        $buf[written] &= 0x7F;

        (&mut $buf[..=written], written + 1)
    }};
}

pub struct VarIntReadFut<R: Unpin + AsyncRead> {
    reader: R,

    size: usize,
    varint: i32,
}

impl<R: Unpin + AsyncRead> Future for VarIntReadFut<R> {
    type Output = Result<(usize, VarInt), ProtoError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let mut buffer = [MaybeUninit::uninit()];
        let mut buffer = ReadBuf::uninit(&mut buffer);

        while Pin::new(&mut self.reader)
            .poll_read(cx, &mut buffer)?
            .is_ready()
        {
            // buffer is only one item long and
            // and buffer gets reset at each loop
            let &[current] = buffer.filled() else {
                return Poll::Ready(Err(ProtoError::Io(ErrorKind::UnexpectedEof.into())));
            };

            // reset the buffer
            buffer.set_filled(0);

            self.varint |= (current.mask_data() as i32) << (self.size * 7);
            self.size += 1;

            // check if byte has stop condition bit or else if
            // it's exceeding its limit return err
            if current.has_stop() {
                return Poll::Ready(Ok((self.size, self.varint.into())));
            } else if self.size >= 5 {
                return Poll::Ready(Err(ProtoError::VarInt));
            }
        }

        Poll::Pending
    }
}

pub trait ReadVarIntExtAsync
where
    Self: Unpin + AsyncRead + Sized,
{
    fn read_varint(&mut self) -> VarIntReadFut<&mut Self> {
        VarIntReadFut {
            reader: self,
            size: 0,
            varint: 0,
        }
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

pub trait WriteVarIntExt
where
    Self: Write,
{
    fn write_varint(&mut self, VarInt(val): VarInt) -> Result<usize, ProtoError> {
        let val = val as u32;
        let mut buf = [0u8; 5];

        let (buf, written) = varintwrite!(buf, val);

        self.write_all(buf).map(|_| written).map_err(Into::into)
    }
}

impl<T: AsyncRead + Unpin> ReadVarIntExtAsync for T {}
impl<T: Read> ReadVarIntExt for T {}
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
