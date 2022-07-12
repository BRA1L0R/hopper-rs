use std::sync::Arc;
use tokio::net::TcpListener;

mod client;
pub mod router;

use crate::protocol::error::ProtoError;
pub use crate::HopperError;
pub use client::Client;
pub use router::Router;

use self::router::Bridge;

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
                let bridge = match router.route(&client).map(Bridge::connect) {
                    Ok(future) => future.await,
                    Err(err) => Err(err),
                };

                match bridge {
                    Ok(bridge) => bridge.bridge(client).await,
                    Err(err) => Err(client.disconnect_err_chain(err).await.into()),
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
