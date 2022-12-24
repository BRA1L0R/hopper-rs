use std::{
    cell::{Cell, RefCell},
    io::Cursor,
};

use super::{
    data::{Deserialize, PacketId},
    VarInt,
};

struct Unknown(());

struct Packet<T> {
    packet_id: VarInt,
    data: Vec<u8>,

    decoded: RefCell<Option<T>>,
}

pub struct IdMismatch;

impl Packet<Unknown> {
    pub fn assert<T: PacketId>(
        Packet {
            packet_id,
            data,
            decoded,
        }: Self,
    ) -> Result<Packet<T>, IdMismatch> {
        // // since Unknown(()) must not be constructed, nor
        // // can it be constructed from outside this crate
        // // then decoded must be None
        // debug_assert!(decoded.is_none());

        if packet_id == T::ID {
            Ok(Packet::<T> {
                packet_id,
                data,
                decoded: Default::default(),
            })
        } else {
            Err(IdMismatch)
        }
    }
}

// impl<T: PacketId + for<'a> Deserialize<Cursor<&'a [u8]>>> Packet<Lazy<T>> {
//     fn data(& self) -> T {
//         self.decoded.

//         match self.decoded {
//             Ok(data) => data,
//             None => T::deserialize(reader),
//         }
//     }
// }

// impl<T: PacketId> TryFrom<Packet<Unknown>> for Packet<T> {
//     type Error = IdMismatch;

//     fn try_from(
//         Packet {
//             packet_id,
//             data,
//             decoded,
//         }: Packet<Unknown>,
//     ) -> Result<Self, Self::Error> {
//         if packet_id == T::ID {
//             Ok(Self {
//                 packet_id,
//                 data,
//                 decoded,
//             })
//         } else {
//             Err(IdMismatch)
//         }
//     }
// }
