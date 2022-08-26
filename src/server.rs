use std::sync::Arc;
use tokio::net::TcpListener;

pub mod bridge;
pub mod client;
pub mod router;

use crate::metrics::{injector::MetricsInjector, EventType, Metrics};
pub use crate::HopperError;
pub use client::IncomingClient;
pub use router::Router;

pub struct Hopper {
    metrics: Arc<Metrics>,
    router: Arc<dyn Router>,
}

impl Hopper {
    pub fn new(router: Arc<dyn Router>, injector: Box<dyn MetricsInjector>) -> Self {
        Self {
            router,
            metrics: Arc::new(Metrics::init(injector)),
        }
    }

    pub async fn listen(&self, listener: TcpListener) -> ! {
        log::info!("Listening on {}", listener.local_addr().unwrap());

        loop {
            let client = listener.accept().await.unwrap();

            // TODO: clone only when needed
            let router = self.router.clone();
            let metrics = self.metrics.clone();

            let handler = async move {
                // receives a handshake from the client and decodes its information
                let mut client = IncomingClient::handshake(client).await?;
                // packets are lazily evaluated, this call evaluates the handshake packet and
                // returns an error if the data received is wrong
                let handshake = client.handshake.data()?;

                // routes a client by reading handshake information
                // then if a route has been found it connects to the server
                // but does not yet send handshaking information
                match router.route(handshake).await {
                    Ok(bridge) => {
                        log::info!("{} connected to {}", client.address, bridge.address()?);

                        let guard =
                            metrics.guard(handshake.server_address.clone(), handshake.next_state);

                        // bridge returns the used traffic in form of bytes
                        // transited from client to server and vice versa
                        guard.send_event(EventType::Connect).await;
                        let bridge_result = bridge.bridge(client).await;
                        guard.send_event(EventType::Disconnect).await;

                        let (serverbound, clientbound) = bridge_result?;
                        guard
                            .send_event(EventType::BandwidthReport {
                                serverbound,
                                clientbound,
                            })
                            .await;

                        log::debug!("Connection terminated, transferred serverbound: {serverbound} bytes clientbound: {clientbound} bytes");
                        Ok(())
                    }
                    Err(err) => {
                        log::error!("Couldn't connect {client}: {err}");

                        client.disconnect_err(&err).await;
                        Err(HopperError::from(err))
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
