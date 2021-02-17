pub mod redis;
// pub mod memory;

use crate::data::*;
use crate::error::TrackerResult;
use async_trait::async_trait;

#[async_trait]
pub trait Storage {
    async fn compaction(&self) -> TrackerResult<()>;
    #[cfg(scrape = "on")]
    async fn scrape(&self, data: &ScrapeRequestData) -> TrackerResult<Option<ScrapeResponseData>>;
    async fn announce(
        &self,
        data: &AnnounceRequestData,
    ) -> TrackerResult<Option<AnnounceResponseData>>;
}
