use server::Server;
use tokio::{main, net::TcpListener};

mod config;
mod server;

#[allow(clippy::uninit_vec)]
pub mod protocol;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:25565").await?;

    // let server = Server::new(;
    // server.listen(listener).await?;

    unreachable!()
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
