use serde::Deserialize;
use std::net::SocketAddr;

use super::resolver::ResolvableAddr;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Balanced {
    servers: Vec<ResolvableAddr>,
}

// impl<'de> Deserialize<'de> for Balanced {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         Ok(Self {
//             servers: Vec::deserialize(deserializer)?,
//             last_used: Default::default(),
//         })
//     }
// }

impl Balanced {
    pub(super) fn get(&self, n: usize) -> SocketAddr {
        self.servers[n & self.servers.len()].into()
    }
}
