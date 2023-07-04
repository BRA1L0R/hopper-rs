use self::injector::{MetricsError, MetricsInjector};
use crate::{protocol::packet_impls::State, server::client::Hostname};
use std::{collections::HashMap, time::Duration};
use tokio::{
    select,
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
    time,
};

pub mod influx;
pub mod injector;

#[derive(Debug)]
pub enum EventType {
    Connect,
    BandwidthReport { serverbound: u64, clientbound: u64 },
    Disconnect, // Disconnect,
}

#[derive(Debug)]
pub struct Event {
    information: GuardInformation,
    event_type: EventType,
}

#[derive(Debug, Clone)]
struct GuardInformation {
    hostname: Hostname,
    state: State,
}

#[derive(Debug)]
pub struct MetricsGuard<'a> {
    information: GuardInformation,
    sender: &'a mpsc::Sender<Event>,
}

impl MetricsGuard<'_> {
    pub async fn send_event(&self, event_type: EventType) {
        self.sender
            .send(Event {
                information: self.information.clone(),
                event_type,
            })
            .await
            .unwrap();
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct HostnameCounter {
    total_pings: u64,
    total_game: u64,

    open_connections: u64,

    serverbound_traffic: u64,
    clientbound_traffic: u64,
}

impl HostnameCounter {
    pub fn apply_event(&mut self, event: &Event) {
        match event.event_type {
            EventType::Connect => {
                match event.information.state {
                    State::Login => self.total_game = self.total_game.wrapping_add(1),
                    State::Status => self.total_pings = self.total_pings.wrapping_add(1),
                }

                self.open_connections = self
                    .open_connections
                    .checked_add(1)
                    .expect("open connections exceeded u64::MAX")
            }
            EventType::BandwidthReport {
                serverbound,
                clientbound,
            } => {
                self.clientbound_traffic = self.clientbound_traffic.wrapping_add(clientbound);
                self.serverbound_traffic = self.serverbound_traffic.wrapping_add(serverbound);
            }
            EventType::Disconnect => {
                self.open_connections = self
                    .open_connections
                    .checked_sub(1)
                    .expect("open connections to be at least 1")
            }
        }
    }
}

#[allow(clippy::mutable_key_type)] // allowed for bytes::Bytes
pub type Counters = HashMap<Hostname, HostnameCounter>;

pub struct Metrics {
    sender: mpsc::Sender<Event>,
    handler: JoinHandle<()>,
}

impl Drop for Metrics {
    fn drop(&mut self) {
        self.handler.abort();
    }
}

impl Metrics {
    pub fn init(injector: Box<dyn MetricsInjector>) -> Self {
        let (sender, receiver) = mpsc::channel::<Event>(8096);

        let handler = tokio::spawn(Metrics::metrics_handler(receiver, injector));

        Self { sender, handler }
    }

    pub fn guard(&self, hostname: Hostname, state: State) -> MetricsGuard {
        MetricsGuard {
            sender: &self.sender,
            information: GuardInformation { hostname, state },
        }
    }

    async fn metrics_handler(mut receiver: Receiver<Event>, injector: Box<dyn MetricsInjector>) {
        let mut counters: Counters = Default::default();

        let mut register_interval = time::interval(Duration::from_secs(5));

        loop {
            let event = select! {
                biased;
                Some(event) = receiver.recv() => event,
                _ = register_interval.tick() => {
                    if let Err(err) = injector.log(&counters).await { log::error!("InfluxDB reported an error: {err}") };
                    continue
                },
            };

            let counter = match counters.get_mut(&event.information.hostname) {
                Some(counter) => counter,
                None => counters
                    .entry(event.information.hostname.clone())
                    .or_default(),
            };

            counter.apply_event(&event)
        }
    }
}
