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
pub mod protocol;
mod server;

// #[allow(clippy::uninit_vec, unused_macros)]
// pub mod protocol;

// only returns a configuration if it's valid
#[cfg(target_os = "linux")]
async fn reload_valid() -> ServerConfig {
    let mut signal =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()).unwrap();

    loop {
        signal.recv().await;

        log::info!("Reloading configuration...");
        match ServerConfig::read() {
            Ok(config) => break config,
            Err(err) => log::error!("Error reloading configuration: {err}"),
        };
    }
}

#[cfg(not(target_os = "linux"))]
fn reload_valid() -> impl futures::Future<Output = ServerConfig> {
    futures::future::pending()
}

async fn run() -> Result<Infallible, HopperError> {
    // reads configuration from Config.toml
    let mut config = ServerConfig::read()?;

    loop {
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
            _ = tokio::signal::ctrl_c() => break Err(HopperError::Signal),
            newconfig = reload_valid() => { config = newconfig },
        }
    }
}

#[main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    let err = run().await.unwrap_err();
    log::error!("{}", err)
}
