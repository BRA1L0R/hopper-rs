use std::{convert::Infallible, sync::Arc};

use crate::config::ServerConfig;
use log::LevelFilter;
use metrics::injector::EmptyInjector;
use server::Hopper;
use simple_logger::SimpleLogger;
use tokio::{main, net::TcpListener};

pub use crate::error::HopperError;

mod config;
pub mod error;
pub mod metrics;
mod server;

#[allow(clippy::uninit_vec, unused_macros)]
pub mod protocol;

async fn run() -> Result<Infallible, HopperError> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    // reads configuration from Config.toml
    let config = ServerConfig::read()?;
    let listener = TcpListener::bind(config.listen)
        .await
        .map_err(HopperError::Bind)?;

    // builds a new hopper instance with a router
    let server = Hopper::new(Arc::new(config.routing), Box::new(EmptyInjector));

    server.listen(listener).await
}

#[main]
async fn main() {
    let err = run().await.unwrap_err();
    log::error!("{}", err)
}
