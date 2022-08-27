use serde::Deserialize;

use crate::metrics::{influx::InfluxInjector, injector::MetricsInjector};

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MetricsConfig {
    #[serde(rename = "influx")]
    Influx {
        /// InfluxDB access url
        url: String,
        /// system hostname
        hostname: Option<String>,

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
                hostname,
                organization,
                token,
                bucket,
            } => {
                let client = influxdb2::Client::new(url, organization, token);

                // use the provided hostname or default to the
                // system hostname, returned in form of a OsString
                // then turned into a rust String
                let hostname = hostname.unwrap_or_else(|| {
                    hostname::get()
                        .expect("can get system hostname")
                        .into_string()
                        .unwrap()
                });

                Box::new(InfluxInjector {
                    client,
                    bucket,
                    hostname,
                })
            }
        }
    }
}
