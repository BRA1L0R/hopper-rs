use serde::Deserialize;

use crate::metrics::{influx::InfluxInjector, injector::MetricsInjector};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MetricsConfig {
    #[serde(rename = "influx")]
    Influx {
        url: String,
        organization: String,
        bucket: String,
        token: String,
    },
}

impl MetricsConfig {
    pub fn injector(self) -> Box<dyn MetricsInjector> {
        match self {
            MetricsConfig::Influx {
                url,
                organization,
                token,
                bucket,
            } => {
                let client = influxdb2::Client::new(url, organization, token);
                Box::new(InfluxInjector { client, bucket })
            }
        }
    }
}
