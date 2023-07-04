use std::ops::Deref;

use super::{Counters, HostnameCounter, MetricsError, MetricsInjector};
use async_trait::async_trait;
use futures::stream;
use influxdb2::models::DataPoint;

pub struct InfluxInjector {
    pub host: String,
    pub bucket: String,
    pub client: influxdb2::Client,
}

#[async_trait]
impl MetricsInjector for InfluxInjector {
    async fn log(&self, counters: &Counters) -> Result<(), MetricsError> {
        let writes: Vec<_> = counters
            .iter()
            .map(|(connecting_host, metrics)| {
                // destructuring ensures that no field will
                // be left out in the future
                let HostnameCounter {
                    total_pings,
                    total_game,
                    open_connections,
                    serverbound_traffic,
                    clientbound_traffic,
                } = *metrics;

                DataPoint::builder("traffic")
                    .tag("host", &self.host)
                    .tag("destination_hostname", connecting_host.deref())
                    .field("total_pings", i64::try_from(total_pings).unwrap())
                    .field("total_game", i64::try_from(total_game).unwrap())
                    .field("open_connections", i64::try_from(open_connections).unwrap())
                    .field(
                        "serverbound_traffic",
                        i64::try_from(serverbound_traffic).unwrap(),
                    )
                    .field(
                        "clientbound_traffic",
                        i64::try_from(clientbound_traffic).unwrap(),
                    )
                    .build()
                    .unwrap()
            })
            .collect();

        self.client
            .write(&self.bucket, stream::iter(writes))
            .await
            .map_err(|err| MetricsError(Box::new(err)))?;

        Ok(())
    }
}
