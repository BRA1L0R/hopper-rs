use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug)]
pub struct Balanced {
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
    pub(super) fn get(&mut self) -> SocketAddr {
        let item = self.servers[self.last_used];
        self.last_used = (self.last_used + 1) % self.servers.len();

        item
    }
}
