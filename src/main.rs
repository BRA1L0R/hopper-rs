use log::LevelFilter;
use server::{ConfigRouter, Hopper};
use simple_logger::SimpleLogger;
use tokio::{main, net::TcpListener};

mod config;
mod server;

#[allow(clippy::uninit_vec, unused_macros)]
pub mod protocol;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let server = Hopper::new(ConfigRouter::new());
    let listener = TcpListener::bind("0.0.0.0:25565").await?;
    server.listen(listener).await?;

    unreachable!()
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
