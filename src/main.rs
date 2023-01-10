use std::{convert::Infallible, sync::Arc};
use std::sync::Mutex;

use crate::config::{metrics::MetricsConfig, ServerConfig};
use log::LevelFilter;
use metrics::injector::EmptyInjector;
use server::Hopper;
use simple_logger::SimpleLogger;
use tokio::{main, net::TcpListener, select};
use tokio::signal::unix::{signal, SignalKind};

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
    let server = Hopper::new(Arc::new(Mutex::new(Arc::new(config.routing))), metrics);

    server.listen_config(signal(SignalKind::hangup()).expect("Unable to get signal"));

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
