use super::{VarInt, VarIntOp};
use crate::protocol::error::ProtoError;

use std::io::Write;

pub trait WriteVarIntExt
where
    Self: Write,
{
    fn write_varint(&mut self, VarInt(val): VarInt) -> Result<usize, ProtoError> {
        let val = val as u32;

        let mut buf = [0u8; 5];
        let mut written = 0;

        // iter until shifted val is zero
        std::iter::successors(Some(val), |val| Some(val >> 7))
            .take_while(|val| *val != 0)
            // add continue bit to every byte, but last
            // byte gets its bit removed
            .map(|val| (val as u8).add_continue())
            .enumerate()
            .for_each(|(pos, val)| {
                written = pos + 1;
                buf[pos] = val
            });
        // remove continue bit from the last element
        buf[written] &= 0x7F;

        self.write_all(&buf[..=written])?;

        Ok(written)
    }
}

impl<T: Write> WriteVarIntExt for T {}
