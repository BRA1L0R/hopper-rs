use std::sync::Arc;
use tokio::net::TcpListener;

mod client;
pub mod router;

pub use crate::HopperError;
pub use client::Client;
pub use router::Router;

pub struct Hopper {
    router: Arc<dyn Router>,
}

impl Hopper {
    pub fn new(router: impl Router + 'static) -> Self {
        let router = Arc::new(router);
        Self { router }
    }

    pub async fn listen(&self, listener: TcpListener) -> ! {
        loop {
            let client = listener.accept().await.unwrap();
            let router = self.router.clone();

            let handler = async move {
                // receives a handshake from the client and decodes its information
                let client = Client::handshake(client).await?;

                // routes a client by reading handshake information
                // then if a route has been found it connects to the server
                // but does not yet send handshaking information
                let route = router.route(&client).await;
                let (server_address, connected_server) = match route {
                    Ok(data) => (data.endpoint().unwrap(), data),
                    Err(err) => {
                        // if routing fails send a reasonable message
                        // to the client
                        client.disconnect(err.to_string()).await;
                        return Err(err.into());
                    }
                };

                log::info!(
                    "Client {:?} accessing {} routed to {server_address:?}",
                    client.address,
                    client.destination()
                );

                // send handshake to the server and bridge the two for
                // further game communication
                connected_server.bridge(client).await
            };

            // creates a new task for each client
            tokio::spawn(async move {
                if let Err(err) = handler.await {
                    log::error!("{}", err)
                };
            });
        }
    }
}
