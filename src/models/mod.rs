use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Uuid,
    pub instance_id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: MemoryMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_count: u64,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub importance: i32,
    pub chain_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub framework: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub threshold: Option<f32>,
    pub category_filter: Option<String>,
    pub tags_filter: Option<Vec<String>>,
    pub instance_filter: Option<Vec<String>>,
    pub hybrid_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub memories: Vec<Memory>,
    pub search_id: Uuid,
    pub query_embedding: Option<Vec<f32>>,
    pub cache_hits: usize,
    pub total_results: usize,
    pub search_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRequest {
    pub content: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub importance: Option<i32>,
    pub chain_id: Option<String>,
    pub parent_id: Option<Uuid>,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreResult {
    pub memory_id: Uuid,
    pub cached: bool,
    pub indexed: bool,
    pub embedding_generated: bool,
}