use super::{EventType, MetricsError, MetricsInjector};
use async_trait::async_trait;
use influxdb::{InfluxDbWriteable, Timestamp};
use std::time::{SystemTime, UNIX_EPOCH};

// impl Event {
//     fn new_query(self) -> WriteQuery {
//         let name = self.event_type.measurement();
//         let query = now()
//             .into_query(name)
//             .add_tag("from", self.from.to_string())
//             .add_tag("hostname", self.hostname);

//             let query = match self.event_type {}

//         match self.event_type {
//             super::EventType::Ping => query.add_field("count", 1),
//             super::EventType::Join { name } | super::EventType::Leave { name } => {
//                 query.add_field("name", name)
//             }
//             super::EventType::BandwidthReport {
//                 name,
//                 server_bound,
//                 client_bound,
//             } => query
//                 .add_field("name", name)
//                 .add_field("server_bound", server_bound)
//                 .add_field("client_bound", client_bound),
//             super::EventType::JoinError { name, error } => todo!(),
//         }
//     }
// }

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
        super::Event {
            from,
            hostname,
            state,
            event_type,
        }: super::Event<'_>,
    ) -> Result<(), MetricsError> {
        let name = event_type.measurement();
        let query = now()
            .into_query(name)
            .add_field("address", from.to_string())
            .add_field("state", state as i32)
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
