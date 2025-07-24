pub mod qdrant_search;
pub mod embeddings;

use crate::error::Result;
use crate::models::{Memory, SearchRequest, SearchResult};
use async_trait::async_trait;

#[async_trait]
pub trait SearchLayer {
    async fn search(&self, request: &SearchRequest) -> Result<SearchResult>;
    async fn index(&self, memory: &Memory) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn update_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()>;
}