use serde::Deserialize;

#[derive(Deserialize)]
pub enum MetricsConfig {
    Influx,
}
