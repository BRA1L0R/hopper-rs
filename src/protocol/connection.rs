use bytes::BytesMut;
use futures::{SinkExt, TryStreamExt};
use netherite::{
    codec::{CodecError, MinecraftCodec},
    encoding::packetid::PacketId,
    packet::RawPacket,
    DeError, Serialize,
};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub type Codec = Framed<TcpStream, MinecraftCodec>;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("invalid packet id")]
    Id,

    #[error("error decoding: {0}")]
    Decode(#[from] DeError),
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("eof")]
    Eof,
    #[error("${0}")]
    Codec(#[from] CodecError),
}

pub struct Connection {
    inner: Codec,
}

impl Connection {
    pub fn new(inner: TcpStream) -> Self {
        Self {
            inner: Codec::new(inner, MinecraftCodec::default()),
        }
    }

    pub fn write_buffer(&mut self) -> &mut BytesMut {
        self.inner.write_buffer_mut()
    }

    pub fn read_buffer(&self) -> &BytesMut {
        self.inner.read_buffer()
    }

    pub fn into_inner(self) -> Codec {
        self.inner
    }

    pub async fn read_packet(&mut self) -> Result<RawPacket, ConnectionError> {
        self.inner.try_next().await?.ok_or(ConnectionError::Eof)
    }

    pub async fn feed_raw_packet(
        &mut self,
        packet: impl AsRef<RawPacket>,
    ) -> Result<(), ConnectionError> {
        let packet = packet.as_ref();
        self.inner.feed(packet).await.map_err(Into::into)
    }

    pub async fn feed_packet<T: PacketId + Serialize>(
        &mut self,
        packet: T,
    ) -> Result<(), ConnectionError> {
        self.inner.feed(packet).await.map_err(Into::into)
    }

    pub async fn flush(&mut self) -> Result<(), ConnectionError> {
        <Codec as SinkExt<&RawPacket>>::flush(&mut self.inner)
            .await
            .map_err(Into::into)
    }
}
