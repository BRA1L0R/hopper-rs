use crate::server::{Client, HopperError, Router};
use config::{ConfigError, File};
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr};

#[derive(Deserialize)]
/// Defines the structure of a config file. Extension can be
pub struct ServerConfig {
    /// listening address
    pub listen: SocketAddr,

    // pub routing: Option<RouterConfig>,
    /// routing configuration
    /// required because no other method is currently supported
    pub routing: RouterConfig,
}

impl ServerConfig {
    pub fn new() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(File::with_name("Config"))
            .build()?
            .try_deserialize()
    }
}

#[derive(Deserialize)]
pub struct RouterConfig {
    routes: HashMap<String, SocketAddr>,
}

impl Router for RouterConfig {
    fn route(&self, client: &Client) -> Result<SocketAddr, HopperError> {
        let destination = client.destination();
        self.routes
            .get(destination)
            .copied()
            .ok_or(HopperError::NoServer)
    }
}
