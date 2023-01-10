use std::{net::SocketAddr, sync::Arc};
use std::sync::Mutex;

use tokio::net::{TcpListener, TcpStream};
use tokio::signal::unix::Signal;

pub use client::IncomingClient;
pub use router::Router;

use crate::{
    metrics::{EventType, injector::MetricsInjector, Metrics},
    server::{backend::Backend, bridge::Bridge},
};
use crate::config::ServerConfig;
pub use crate::HopperError;

mod backend;
pub mod bridge;
pub mod client;
pub mod router;

macro_rules! try_client {
    ($v:expr, $client:expr, $message:tt) => {
        match $v {
            Ok(v) => v,
            Err(err) => {
                log::error!($message, err);

                $client.disconnect_err(&err).await;
                return Err(err.into());
            }
        }
    };
}

pub struct Hopper {
    metrics: Arc<Metrics>,
    router: Arc<Mutex<Arc<dyn Router>>>,
}

impl Hopper {
    pub fn new(router: Arc<Mutex<Arc<dyn Router>>>, injector: Box<dyn MetricsInjector>) -> Self {
        Self {
            router,
            metrics: Arc::new(Metrics::init(injector)),
        }
    }

    pub async fn handler(
        client: (TcpStream, SocketAddr),
        router: Arc<dyn Router>,
        metrics: Arc<Metrics>,
    ) -> Result<(), HopperError> {
        // receives a handshake from the client and decodes its information
        let mut client = IncomingClient::init(client).await?;

        // routes a client by reading handshake information
        // then if a route has been found it connects to the server
        // but does not yet send handshaking information
        let route = try_client!(
            router.route(&mut client),
            client,
            "Couldn't route {client}: {}"
        );

        let route_addr = route.address();
        log::info!("connecting {} to {route_addr}", client.address);

        let backend = try_client!(
            Backend::connect(&route).await,
            client,
            "Cannot connect {client} to {route_addr}: {}"
        );

        // create a metricsguard which contains a channel where
        // events are sent, and then added to the metrics state
        let guard = metrics.guard(client.hostname.clone(), client.handshake.data().next_state);

        let bridge = Bridge::new(backend, client, route.strategy());

        // bridge returns the used traffic in form of bytes
        // transited from client to server and vice versa
        guard.send_event(EventType::Connect).await;
        // let bridge_result = route.bridge(client).await;
        let bridge_result = bridge.bridge().await;
        guard.send_event(EventType::Disconnect).await;

        // this result is evaluated later so disconnections are
        // always registered no matter the bridge outcome
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

    pub async fn listen(&self, listener: TcpListener) -> ! {
        log::info!("Listening on {}", listener.local_addr().unwrap());

        loop {
            let client = listener.accept().await.unwrap();

            // cheap to clone but it'd be better to clone only if needed
            // TODO: clone only when needed
            let router = self.router.clone().lock().unwrap().clone();
            let metrics = self.metrics.clone();
            // creates a new task for each client
            tokio::spawn(async move {
                if let Err(err) = Self::handler(client, router, metrics).await {
                    log::debug!("{}", err)
                };
            });

            // yield execution to the executor
            // because accepting sockets might get
            // into a tight loop and monopolize cpu
            tokio::task::yield_now().await
        }
    }

    pub(crate) fn listen_config(&self, mut stream: Signal) {
        let router = self.router.clone();
        tokio::spawn(async move {
            loop {
                stream.recv().await;

                let config = ServerConfig::read().expect("Unable to read config");

                let mut rt = router.lock().unwrap();
                *rt = Arc::new(config.routing);

                log::info!("Reloaded routing.")
            }
        });
    }
}
