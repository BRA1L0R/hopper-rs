use super::{EventType, MetricsError, MetricsInjector};
use async_trait::async_trait;
use influxdb::{InfluxDbWriteable, Timestamp};
use std::time::{SystemTime, UNIX_EPOCH};

impl EventType {
    pub fn measurement(&self) -> &'static str {
        match self {
            EventType::BandwidthReport { .. } => "bandwidth",
            EventType::Connect | EventType::ConnectionError { .. } | EventType::Disconnect => {
                "player_activity"
            }
        }
    }
}

fn now() -> Timestamp {
    Timestamp::Milliseconds(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
}

pub struct InfluxInjector {
    client: influxdb::Client,
}

impl InfluxInjector {
    pub fn new(url: impl Into<String>, database: impl Into<String>) -> Self {
        let client = influxdb::Client::new(url, database);
        Self { client }
    }
}

#[async_trait]
impl MetricsInjector for InfluxInjector {
    async fn log(
        &self,
        super::Metric {
            from,
            hostname,
            event_type,
        }: super::Metric<'_>,
    ) -> Result<(), MetricsError> {
        let name = event_type.measurement();
        let query = now()
            .into_query(name)
            .add_field("address", from.to_string())
            .add_tag("hostname", hostname);

        let query = match event_type {
            super::EventType::Connect => query.add_tag("type", "join"),
            super::EventType::Disconnect => query.add_tag("type", "leave"),
            super::EventType::ConnectionError { error } => {
                query.add_tag("type", "error").add_field("error", error)
            }

            super::EventType::BandwidthReport {
                server_bound,
                client_bound,
            } => query
                .add_field("server_bound", server_bound)
                .add_field("client_bound", client_bound),
        };

        self.client
            .query(query)
            .await
            .map_err(|err| MetricsError(Box::new(err)))
            .map(drop)
    }
}
