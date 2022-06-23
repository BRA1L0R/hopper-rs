use self::router::Server;
use std::{convert::Infallible, sync::Arc};
use tokio::{io, net::TcpListener};

mod client;
mod error;
mod router;

pub use client::Client;
pub use error::HopperError;
pub use router::Router;

pub struct Hopper {
    router: Arc<dyn Router>,
}

impl Hopper {
    pub fn new<R: Router + 'static>(router: R) -> Self {
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
