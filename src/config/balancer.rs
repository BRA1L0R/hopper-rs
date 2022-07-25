use serde::Deserialize;
use std::net::SocketAddr;

use super::resolver::ResolvableAddr;

#[derive(Debug)]
pub struct Balanced {
    servers: Vec<ResolvableAddr>,
    last_used: usize,
}

impl<'de> Deserialize<'de> for Balanced {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            servers: Vec::deserialize(deserializer)?,
            last_used: Default::default(),
        })
    }
}

impl Balanced {
    pub(super) fn get(&mut self) -> SocketAddr {
        let item = self.servers[self.last_used];
        self.last_used = (self.last_used + 1) % self.servers.len();

        item.into()
    }
}
