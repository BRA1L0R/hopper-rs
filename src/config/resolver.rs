use std::net::{SocketAddr, ToSocketAddrs};

use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone, Copy)]
pub(super) struct ResolvableAddr(#[serde(deserialize_with = "resolve_hostname")] SocketAddr);

fn resolve_hostname<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let inner = <String>::deserialize(deserializer)?;

    let mut addr = inner
        .to_socket_addrs()
        .map_err(|err| Error::custom(format!("invalid hostname format: {err}")))?;

    addr.next().ok_or_else(|| Error::custom("msg"))
}

impl From<ResolvableAddr> for SocketAddr {
    fn from(addr: ResolvableAddr) -> Self {
        addr.0
    }
}
