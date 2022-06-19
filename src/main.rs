use crate::protocol::{packets::Handshake, PacketExtAsync};
use protocol::error::ProtoError;
use tokio::{
    main,
    net::{TcpListener, TcpStream},
};

pub mod error;

#[allow(clippy::uninit_vec)]
mod protocol;
mod server;

async fn handle_client(mut stream: TcpStream) -> Result<(), ProtoError> {
    let handshake = stream
        .read_packet()
        .await?
        .deserialize_owned::<Handshake>()?;

    println!("{handshake:?}");

    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:25565").await?;

    while let (mut stream, _) = listener.accept().await? {
        tokio::spawn(handle_client(stream));
    }

    todo!()
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
