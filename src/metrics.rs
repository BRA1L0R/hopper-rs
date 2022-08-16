use crate::protocol::packets::State;
use async_trait::async_trait;
use std::net::SocketAddr;
use thiserror::Error;

pub mod api;
pub mod influx;

#[derive(Debug)]
pub enum EventType {
    Connect,
    Disconnect,
    ConnectionError {
        error: String,
    },
    BandwidthReport {
        server_bound: u64,
        client_bound: u64,
    },
}

#[derive(Debug)]
pub struct Event<'a> {
    from: &'a SocketAddr,
    hostname: &'a str,
    state: State,

    event_type: EventType,
}

impl<'a> Event<'a> {
    pub fn new(
        from: &'a SocketAddr,
        hostname: &'a str,
        state: State,
        event: EventType,
    ) -> Event<'a> {
        Self {
            from,
            hostname,
            state,
            event_type: event,
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct MetricsError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>);

#[async_trait]
pub trait MetricsInjector: Send + Sync {
    async fn log(&self, event: Event<'_>) -> Result<(), MetricsError>;
}

pub struct EmptyInjector;

#[async_trait]
impl MetricsInjector for EmptyInjector {
    async fn log(&self, _: Event<'_>) -> Result<(), MetricsError> {
        Ok(())
    }
}
