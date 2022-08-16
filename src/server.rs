use std::sync::Arc;
use tokio::net::TcpListener;

pub mod bridge;
pub mod client;
pub mod router;

use crate::metrics::{api::MetricsApi, MetricsInjector};
pub use crate::HopperError;
pub use client::IncomingClient;
pub use router::Router;

pub struct Hopper {
    metrics: Arc<dyn MetricsInjector>,
    router: Arc<dyn Router>,
}

impl Hopper {
    pub fn new(metrics: Arc<dyn MetricsInjector>, router: Arc<dyn Router>) -> Self {
        Self { router, metrics }
    }

    pub async fn listen(&self, listener: TcpListener) -> ! {
        log::info!("Listening on {}", listener.local_addr().unwrap());

        loop {
            let client = listener.accept().await.unwrap();
            let router = self.router.clone();
            let metrics = self.metrics.clone();

            let handler = async move {
                // receives a handshake from the client and decodes its information
                let mut client = IncomingClient::handshake(client).await?;
                let handshake = client.handshake.data()?;

                // construct a metrics guard with the parameters
                // of the current connection
                let metrics = MetricsApi::new(
                    metrics,
                    client.address,
                    &handshake.server_address,
                    handshake.next_state,
                );

                // routes a client by reading handshake information
                // then if a route has been found it connects to the server
                // but does not yet send handshaking information
                match router.route(handshake).await {
                    Ok(bridge) => {
                        log::info!("{client} connected to {}", bridge.address()?);
                        metrics.join().await?;

                        bridge.bridge(client).await
                    }
                    Err(err) => {
                        log::error!("Couldn't connect {client}: {err}");
                        metrics.join_error(&err).await?;

                        let err = client
                            .disconnect_err_chain(HopperError::Router(Box::new(err)))
                            .await;
                        Err(err)
                    }
                }
            };

            // creates a new task for each client
            tokio::spawn(async move {
                if let Err(err) = handler.await {
                    log::debug!("{}", err)
                };
            });
        }
    }
}
