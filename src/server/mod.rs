use self::{
    client::Client,
    router::{Router, Server},
};
use error::HopperError;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{io, net::TcpListener};

mod client;
mod error;
mod router;

pub struct Hopper<R> {
    router: Arc<R>,
}

impl<R> Hopper<R>
where
    R: 'static + Router + Sync,
{
    pub fn new(router: R) -> Self {
        let router = Arc::new(router);
        Self { router }
    }

    pub async fn listen(&self, listener: TcpListener) -> Result<Infallible, io::Error> {
        loop {
            let client = listener.accept().await?;
            let router = self.router.clone();

            tokio::spawn(async move {
                let client = Client::handshake(client).await?;

                let server_address = router.route(&client)?;
                let server = Server::connect(server_address).await?;

                log::info!(
                    "Client {:?} accessing {} routed to {server_address:?}",
                    client.address,
                    client.destination()
                );

                server.bridge(client).await
            });
        }
    }
}

pub struct ConfigRouter {
    routes: HashMap<String, SocketAddr>,
}

impl ConfigRouter {
    pub fn new() -> Self {
        let mut routes = HashMap::new();

        routes.insert(
            String::from("localhost"),
            SocketAddr::from_str("10.1.244.99:25008").unwrap(),
        );

        Self { routes }
    }
}

impl Router for ConfigRouter {
    fn route(&self, client: &Client) -> Result<SocketAddr, HopperError> {
        let destination = client.destination();
        self.routes
            .get(destination)
            .copied()
            .ok_or(HopperError::NoServer)
    }
}
