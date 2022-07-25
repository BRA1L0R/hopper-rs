use std::sync::Arc;
use tokio::net::TcpListener;

pub mod bridge;
mod client;
pub mod router;

pub use crate::HopperError;
pub use client::IncomingClient;
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
        log::info!("Listening on {}", listener.local_addr().unwrap());

        loop {
            let client = listener.accept().await.unwrap();
            let router = self.router.clone();

            let handler = async move {
                // receives a handshake from the client and decodes its information
                let mut client = IncomingClient::handshake(client).await?;
                let handshake = client.handshake.data()?;

                // routes a client by reading handshake information
                // then if a route has been found it connects to the server
                // but does not yet send handshaking information
                match router.route(handshake).await {
                    Ok(bridge) => {
                        log::info!("{client} connected to {}", bridge.address()?);
                        bridge.bridge(client).await
                    }
                    Err(err) => {
                        log::error!("Couldn't connect {client}: {err}");
                        Err(client.disconnect_err_chain(err.into()).await)
                    }
                }
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
