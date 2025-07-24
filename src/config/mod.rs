use serde::{Deserialize, Serialize};
use crate::error::{Result, UnifiedRagError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
    pub qdrant: QdrantConfig,
    pub openai: OpenAIConfig,
    pub instance_id: String,
    pub cache_ttl_seconds: u64,
    pub max_results: usize,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub pool_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub host: String,
    pub port: u16,
    pub collection_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            redis: RedisConfig {
                host: std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("REDIS_PORT")
                    .unwrap_or_else(|_| "6379".to_string())
                    .parse()
                    .unwrap_or(6379),
                password: std::env::var("REDIS_PASSWORD").ok(),
                pool_size: 10,
            },
            qdrant: QdrantConfig {
                host: std::env::var("QDRANT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("QDRANT_PORT")
                    .unwrap_or_else(|_| "6333".to_string())
                    .parse()
                    .unwrap_or(6333),
                collection_name: std::env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_| "unified_rag".to_string()),
            },
            openai: OpenAIConfig {
                api_key: std::env::var("OPENAI_API_KEY")
                    .map_err(|_| UnifiedRagError::Configuration("OPENAI_API_KEY not set".to_string()))?,
                model: "text-embedding-3-small".to_string(),
            },
            instance_id: std::env::var("INSTANCE_ID").unwrap_or_else(|_| "CC".to_string()),
            cache_ttl_seconds: 3600, // 1 hour default
            max_results: 20,
            similarity_threshold: 0.7,
        })
    }
}