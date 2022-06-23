use std::convert::Infallible;

use crate::config::ServerConfig;
use log::LevelFilter;
use server::Hopper;
use simple_logger::SimpleLogger;
use tokio::{main, net::TcpListener};

mod config;
mod server;

#[allow(clippy::uninit_vec, unused_macros)]
pub mod protocol;

async fn run() -> Result<Infallible, Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    // reads configuration from Config.toml
    let config = ServerConfig::new()?;

    let listener = TcpListener::bind(config.listen).await?;
    let server = Hopper::new(config.routing);
    server.listen(listener).await.map_err(Into::into)
}

#[main]
async fn main() -> Result<Infallible, Box<dyn std::error::Error>> {
    run().await
}
