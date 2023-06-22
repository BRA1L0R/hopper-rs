use super::error::ProtoError;
use std::io::{Read, Write};

/// defines the packet id of a Minecraft packet
pub trait PacketId {
    const ID: i32; // same type as VarInt
}

/// Trait for sync deserialization of packet
/// data buffers
///
/// It is implemented by both entire packets and
/// singular piece of informations contained in the
/// packet. Structs can derive this macro with
/// the local hopper-macro crate, which sequentially
/// deserializes struct fields in the order they present
pub trait Deserialize: Sized {
    /// deserializes a block of data off a data buffer
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, ProtoError>;
}

/// Trait for sync serialization of packets
///
/// Refer to [`Deserialize`] for where to find the
/// derive macro
pub trait Serialize: Sized + Send {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), ProtoError>;

    /// used as a help for creating a buffer where to write
    /// this packet, or else the buffer would have to dynamically
    /// grow and reallocate too frequently
    ///
    /// to be effective, this value SHOULD represents
    /// the minimum space this structure will take
    /// in the write buffer
    ///
    /// default implementation set to 0 as not every length can
    /// be known efficiently
    fn min_size(&self) -> usize {
        0
    }
}
