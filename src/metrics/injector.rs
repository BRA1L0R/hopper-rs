use async_trait::async_trait;
use thiserror::Error;

use super::Counters;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct MetricsError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>);

#[async_trait]
pub trait MetricsInjector: Send + Sync {
    async fn log(&self, counters: &Counters) -> Result<(), MetricsError>;
}

pub struct EmptyInjector;

#[async_trait]
impl MetricsInjector for EmptyInjector {
    async fn log(&self, counters: &Counters) -> Result<(), MetricsError> {
        log::debug!("EmptyInjector received logs: {counters:?}");
        Ok(())
    }
}
