use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnifiedRagError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Redis pool error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),
    
    #[error("Qdrant error: {0}")]
    Qdrant(String),
    
    #[error("OpenAI error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),
    
    #[error("Invalid configuration: {0}")]
    Configuration(String),
    
    #[error("Cache miss for key: {0}")]
    CacheMiss(String),
    
    #[error("Search failed: {0}")]
    SearchError(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Tool execution error: {0}")]
    ToolError(String),
    
    #[error("Invalid session: {0}")]
    InvalidSession(String),
}

pub type Result<T> = std::result::Result<T, UnifiedRagError>;