use std::{collections::HashMap, net::SocketAddr, ops::Deref};

use serde::Deserialize;

use crate::server::{
    bridge::forwarding::ForwardStrategy,
    router::{Destination, RouterError},
    IncomingClient, Router,
};

use self::{balancer::Balanced, resolver::ResolvableAddr};

mod balancer;
mod resolver;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RouteType {
    Simple(ResolvableAddr),
    // #[serde(deserialize_with = "deserialize_mutex")]
    Balanced(Balanced),
}

// impl RouteType {
//     async fn get(&self) -> SocketAddr {
//         match self {
//             RouteType::Simple(route) => (*route).into(),
//             RouteType::Balanced(balancer) => balancer.get,
//         }
//     }
// }

#[derive(Deserialize, Debug)]
pub struct RouteInfo {
    #[serde(alias = "ip-forwarding", default)]
    ip_forwarding: ForwardStrategy,

    ip: RouteType,
}

#[derive(Deserialize, Debug)]
pub struct RouterConfig {
    default: Option<RouteInfo>,

    #[serde(default)]
    routes: HashMap<String, RouteInfo>,
}

// #[async_trait::async_trait]
impl Router for RouterConfig {
    // type Error = ConfigRouterError;

    fn route(&self, client: &mut IncomingClient) -> Result<Destination, RouterError> {
        // resolve hostname from the configuration
        let route = self
            .routes
            .get(client.hostname.deref())
            .or(self.default.as_ref())
            .ok_or(RouterError::NoServer)?;

        let address: SocketAddr = match route.ip {
            RouteType::Simple(address) => address.into(),
            RouteType::Balanced(ref list) => {
                let hash = client.hash();
                list.get(hash as usize)
            }
        };

        Ok(Destination::new(address, route.ip_forwarding))
    }
}
