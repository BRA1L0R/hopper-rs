use std::{
    io::{ErrorKind, Read},
    mem::MaybeUninit,
    pin::Pin,
    task::Poll,
};

use byteorder::ReadBytesExt;
use futures::Future;
use tokio::io::{AsyncRead, ReadBuf};

use crate::protocol::error::ProtoError;

use super::{VarInt, VarIntOp};

pub trait ReadVarIntExtAsync
where
    Self: Unpin + AsyncRead + Sized,
{
    fn read_varint(&mut self) -> VarIntReadFut<&mut Self> {
        VarIntReadFut::new(self)
    }
}

pub trait ReadVarIntExt
where
    Self: Read,
{
    fn read_varint(&mut self) -> Result<(usize, VarInt), ProtoError> {
        let mut res = 0;

        // max 5 bytes (0 included, 5 excluded)
        for pos in 0..5 {
            let current_byte = self.read_u8()?;
            res |= ((current_byte.mask_data()) as i32) << (pos * 7);

            if current_byte.has_stop() {
                return Ok((pos + 1, VarInt(res)));
            }
        }

        Err(ProtoError::VarInt)
    }
}

pub struct VarIntReadFut<R: Unpin + AsyncRead> {
    reader: R,

    size: usize,
    varint: i32,
}

impl<R: Unpin + AsyncRead> VarIntReadFut<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            size: 0,
            varint: 0,
        }
    }
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

        // if reader not ready then tell
        // the caller the future must wait.
        // cx has already been taking by
        // poll_read
        Poll::Pending
    }
}

impl<T: AsyncRead + Unpin> ReadVarIntExtAsync for T {}
impl<T: Read> ReadVarIntExt for T {}
