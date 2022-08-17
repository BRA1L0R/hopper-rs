use crate::{
    protocol::packets::Handshake,
    server::{
        bridge::{Bridge, ForwardStrategy},
        router::RouterError,
        Router,
    },
};
use async_trait::async_trait;
use config::{ConfigError, File};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

use self::{balancer::Balanced, resolver::ResolvableAddr};

mod balancer;
mod resolver;

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

fn deserialize_mutex<'de, D, T: Deserialize<'de>>(deserializer: D) -> Result<Mutex<T>, D::Error>
where
    D: Deserializer<'de>,
{
    let inner = T::deserialize(deserializer)?;
    Ok(Mutex::new(inner))
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RouteType {
    Simple(ResolvableAddr),
    #[serde(deserialize_with = "deserialize_mutex")]
    Balanced(Mutex<Balanced>),
}

impl RouteType {
    async fn get(&self) -> SocketAddr {
        match self {
            RouteType::Simple(route) => (*route).into(),
            RouteType::Balanced(balancer) => balancer.lock().await.get(),
        }
    }
}

#[derive(Deserialize)]
pub struct RouteInfo {
    #[serde(alias = "ip-forwarding", default)]
    ip_forwarding: ForwardStrategy,

    ip: RouteType,
}

#[derive(Deserialize)]
pub struct RouterConfig {
    default: Option<RouteInfo>,

    #[serde(default)]
    routes: HashMap<String, RouteInfo>,
}

#[async_trait]
impl Router for RouterConfig {
    // type Error = ConfigRouterError;

    async fn route(&self, handshake: &Handshake) -> Result<Bridge, RouterError> {
        let destination = &handshake.server_address;

        // resolve hostname from the configuration
        let route = self
            .routes
            .get(destination)
            .or(self.default.as_ref())
            .ok_or(RouterError::NoServer)?;

        // connect to the resolved backend server
        Bridge::connect(route.ip.get().await, route.ip_forwarding)
            .await
            .map_err(RouterError::Unreachable)
    }
}
