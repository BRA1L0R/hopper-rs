use std::{net::SocketAddr, sync::Arc};

use super::{Event, EventType, MetricsError, MetricsInjector};
use crate::protocol::packets::State;

pub struct MetricsApi {
    injector: Arc<dyn MetricsInjector>,

    client_address: SocketAddr,
    hostname: String,
    state: State,
}

impl MetricsApi {
    pub fn new(
        injector: Arc<dyn MetricsInjector>,
        client_address: SocketAddr,
        hostname: impl Into<String>,
        state: State,
    ) -> Self {
        Self {
            injector,
            client_address,
            hostname: hostname.into(),
            state,
        }
    }

    pub async fn join(&self) -> Result<(), MetricsError> {
        self.log_event(EventType::Connect).await
    }

    pub async fn join_error(self, error: impl std::error::Error) -> Result<(), MetricsError> {
        self.log_event(EventType::ConnectionError {
            error: error.to_string(),
        })
        .await
    }

    pub async fn disconnect(self) -> Result<(), MetricsError> {
        self.log_event(EventType::Disconnect).await
    }

    async fn log_event(&self, event_type: EventType) -> Result<(), MetricsError> {
        let event = Event {
            from: &self.client_address,
            hostname: &self.hostname,
            state: self.state,
            event_type,
        };

        self.injector
            .log(event)
            .await
            .map_err(|err| MetricsError(err.into()))
    }
}
