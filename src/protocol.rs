// mod deserialize;
pub mod data;
pub mod error;
pub mod packets;

pub mod connection;
pub mod packet;
pub mod uuid;
pub mod varint;
pub use varint::VarInt;
pub mod lazy;

const SIZE_LIMIT: usize = 2 * 1024 * 1024; // 2 megabytes
