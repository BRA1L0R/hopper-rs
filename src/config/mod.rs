use crate::server::{bridge::Bridge, router::RouterError, Client, Router};
use async_trait::async_trait;
use config::{ConfigError, File};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

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

struct Balanced {
    servers: Vec<SocketAddr>,
    last_used: usize,
}

impl<'de> Deserialize<'de> for Balanced {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let servers = Vec::deserialize(deserializer)?;
        Ok(Self {
            servers,
            last_used: Default::default(),
        })
    }
}

impl Balanced {
    fn get(&mut self) -> SocketAddr {
        let item = self.servers[self.last_used];
        self.last_used = (self.last_used + 1) % self.servers.len();

        item
    }
}

fn deserialize_mutex<'de, D, T: Deserialize<'de>>(deserializer: D) -> Result<Mutex<T>, D::Error>
where
    D: Deserializer<'de>,
{
    let inner = T::deserialize(deserializer)?;
    Ok(Mutex::new(inner))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RouteType {
    Simple(SocketAddr),
    #[serde(deserialize_with = "deserialize_mutex")]
    Balanced(Mutex<Balanced>),
}

impl RouteType {
    async fn get(&self) -> SocketAddr {
        match self {
            RouteType::Simple(route) => *route,
            RouteType::Balanced(balancer) => balancer.lock().await.get(),
        }
    }
}

#[derive(Deserialize)]
pub struct RouterConfig {
    default: Option<RouteType>,

    #[serde(default)]
    routes: HashMap<String, RouteType>,
}

#[async_trait]
impl Router for RouterConfig {
    async fn route(&self, client: &Client) -> Result<Bridge, RouterError> {
        let destination = client.destination();
        let route = self
            .routes
            .get(destination)
            .or(self.default.as_ref())
            .ok_or(RouterError::NoServer)?;

        Bridge::connect(route.get().await).await
    }
}
