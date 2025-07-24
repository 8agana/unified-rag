pub mod redis_cache;

use crate::error::Result;
use crate::models::{Memory, SearchRequest};
use async_trait::async_trait;

#[async_trait]
pub trait CacheLayer {
    async fn get(&self, key: &str) -> Result<Option<Memory>>;
    async fn set(&self, key: &str, memory: &Memory, ttl_seconds: Option<u64>) -> Result<()>;
    async fn search_cached(&self, request: &SearchRequest) -> Result<Vec<Memory>>;
    async fn invalidate(&self, key: &str) -> Result<()>;
    async fn get_stats(&self) -> Result<CacheStats>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_keys: u64,
    pub memory_usage_bytes: u64,
    pub hit_rate: f32,
    pub miss_rate: f32,
    pub avg_retrieval_time_ms: f32,
}

use serde::{Deserialize, Serialize};