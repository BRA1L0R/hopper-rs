use crate::server::{
    router::{ConnectedServer, RouterError},
    Client, Router,
};
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
    /// reads configuration from Config.toml
    /// (more file exts can be supported through config's features)
    pub fn new() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(File::with_name("Config"))
            .build()?
            .try_deserialize()
    }
}

#[derive(Deserialize)]
pub struct RouterConfig {
    default: Option<SocketAddr>,
    routes: HashMap<String, SocketAddr>,
}

#[async_trait::async_trait]
impl Router for RouterConfig {
    async fn route(&self, client: &Client) -> Result<ConnectedServer, RouterError> {
        let destination = client.destination();
        self.routes
            // tries to read from hashmap
            .get(destination)
            // if not present, uses the optional default
            .or(self.default.as_ref())
            .copied()
            // in case both return None
            .ok_or(RouterError::NoServer)
            // create a future which connects but does not
            // instanciate a minecraft session
            .map(ConnectedServer::connect)?
            .await
    }
}
