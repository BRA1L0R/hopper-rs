use std::{convert::Infallible, sync::Arc};

use crate::config::{metrics::MetricsConfig, ServerConfig};
use log::LevelFilter;
use metrics::injector::EmptyInjector;
use server::Hopper;
use simple_logger::SimpleLogger;
use tokio::{main, net::TcpListener, select};

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

    let metrics = config
        .metrics
        .map(MetricsConfig::injector)
        .unwrap_or_else(|| Box::new(EmptyInjector));

    // builds a new hopper instance with a router
    let server = Hopper::new(Arc::new(config.routing), metrics);

    select! {
        _ = server.listen(listener) => unreachable!(),
        _ = tokio::signal::ctrl_c() => Err(HopperError::Signal),
    }
}

#[main]
async fn main() {
    let err = run().await.unwrap_err();
    log::error!("{}", err)
}
