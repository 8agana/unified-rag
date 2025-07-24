use anyhow::Result;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{CallToolResult, Content, ErrorData},
    ServerHandler,
};
use rmcp_macros::{tool, tool_router, tool_handler};
use std::sync::Arc;
use std::future::Future;
use deadpool_redis::{Config as RedisConfig, Runtime};
use qdrant_client::Qdrant;
use crate::tools::{RagSearchParams, RagStoreParams};
use crate::cache::{CacheLayer, redis_cache::RedisCache};
use crate::search::{SearchLayer, qdrant_search::QdrantSearch, embeddings::EmbeddingGenerator};
use crate::models::SearchRequest;

#[derive(Clone)]
pub struct UnifiedRagService {
    tool_router: ToolRouter<Self>,
    redis_pool: Arc<deadpool_redis::Pool>,
    qdrant_client: Arc<Qdrant>,
    cache: Arc<RedisCache>,
    search: Arc<QdrantSearch>,
    instance_id: String,
}

impl UnifiedRagService {
    pub async fn new() -> Result<Self> {
        // Load configuration from environment
        let redis_host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let redis_port = std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_password = std::env::var("REDIS_PASSWORD").ok();
        let instance_id = std::env::var("INSTANCE_ID").unwrap_or_else(|_| "CC".to_string());
        
        // Configure Redis pool
        let redis_url = if let Some(password) = redis_password {
            format!("redis://:{}@{}:{}/0", password, redis_host, redis_port)
        } else {
            format!("redis://{}:{}/0", redis_host, redis_port)
        };
        
        let cfg = RedisConfig::from_url(redis_url);
        let redis_pool = Arc::new(cfg.create_pool(Some(Runtime::Tokio1))?);
        
        // Configure Qdrant client
        let qdrant_host = std::env::var("QDRANT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let qdrant_port = std::env::var("QDRANT_PORT")
            .unwrap_or_else(|_| "6334".to_string())
            .parse::<u16>()
            .unwrap_or(6334);
        
        // Use the Qdrant protocol from env var, defaulting to http for local instances
        let qdrant_protocol = std::env::var("QDRANT_PROTOCOL").unwrap_or_else(|_| "http".to_string());
        let qdrant_url = format!("{}://{}:{}", qdrant_protocol, qdrant_host, qdrant_port);
        
        tracing::info!("Connecting to Qdrant at: {}", qdrant_url);
        
        // Create Qdrant client with custom configuration
        // Use port 6333 for HTTP API (not gRPC which uses 6334)
        let qdrant_client = Arc::new(
            match Qdrant::from_url(&qdrant_url)
                .timeout(std::time::Duration::from_secs(30))
                .build() {
                Ok(client) => client,
                Err(e) => {
                    tracing::error!("Failed to create Qdrant client: {}", e);
                    // Try with explicit HTTP/1.1 if it's a protocol error
                    if e.to_string().contains("h2") || e.to_string().contains("HTTP/2") {
                        tracing::info!("Retrying Qdrant connection with HTTP/1.1");
                        // For now, we'll just fail - the proper fix would require
                        // using a custom reqwest client with http1_only()
                        return Err(anyhow::anyhow!(
                            "Qdrant H2 protocol error. Please ensure Qdrant is configured for HTTP/1.1 or use gRPC port 6334"
                        ));
                    }
                    return Err(anyhow::anyhow!("Failed to create Qdrant client: {}", e));
                }
            }
        );
        
        // Initialize cache and search layers
        let cache = Arc::new(RedisCache::new(redis_pool.clone(), &instance_id));
        
        // Try to create embedding generator
        let embedding_generator = match EmbeddingGenerator::new() {
            Ok(eg) => Arc::new(eg),
            Err(e) => {
                tracing::error!("Failed to create embedding generator: {}. Some features may be unavailable.", e);
                return Err(e.into());
            }
        };
        
        let collection_name = std::env::var("QDRANT_COLLECTION")
            .unwrap_or_else(|_| "unified_rag".to_string());
        
        // Try to initialize Qdrant search
        let search = match QdrantSearch::new(qdrant_client.clone(), collection_name, embedding_generator).await {
            Ok(s) => Arc::new(s),
            Err(e) => {
                tracing::error!("Failed to initialize Qdrant search layer: {}", e);
                return Err(e.into());
            }
        };
        
        tracing::info!(
            "UnifiedRAG service initialized - Instance: {}, Redis: {}:{}, Qdrant: {}",
            instance_id, redis_host, redis_port, qdrant_url
        );
        
        Ok(Self {
            tool_router: Self::tool_router(),
            redis_pool,
            qdrant_client,
            cache,
            search,
            instance_id,
        })
    }
}

/// Implementation of MCP tools using rmcp macros
#[tool_router]
impl UnifiedRagService {
    /// Search for memories using hybrid L1/L2 retrieval
    #[tool(description = "Search for memories using hybrid L1/L2 retrieval with Redis caching and Qdrant semantic search")]
    pub async fn rag_search(
        &self,
        params: Parameters<RagSearchParams>,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        let request = SearchRequest {
            query: params.0.query,
            limit: Some(params.0.limit),
            threshold: Some(params.0.threshold),
            category_filter: params.0.category_filter,
            tags_filter: params.0.tags_filter,
            instance_filter: params.0.instance_filter,
            hybrid_mode: params.0.hybrid_mode,
        };
        
        // Try cache first if hybrid mode
        let result = if request.hybrid_mode {
            match self.cache.search_cached(&request).await {
                Ok(cache_results) if !cache_results.is_empty() => {
                    // Return cache results
                    serde_json::json!({
                        "memories": cache_results,
                        "source": "cache",
                        "count": cache_results.len()
                    })
                }
                Ok(_) | Err(_) => {
                    // Fall back to Qdrant (empty cache results or cache error)
                    match self.search.search(&request).await {
                        Ok(search_result) => {
                            serde_json::to_value(search_result)
                                .map_err(|e| ErrorData::internal_error(format!("Failed to serialize search results: {}", e), None))?
                        }
                        Err(e) => {
                            tracing::error!("Search failed: {}", e);
                            return Err(ErrorData::internal_error(
                                format!("Search failed: {}. Please check that Qdrant is running and accessible.", e),
                                None
                            ));
                        }
                    }
                }
            }
        } else {
            // Direct Qdrant search
            match self.search.search(&request).await {
                Ok(search_result) => {
                    serde_json::to_value(search_result)
                        .map_err(|e| ErrorData::internal_error(format!("Failed to serialize search results: {}", e), None))?
                }
                Err(e) => {
                    tracing::error!("Search failed: {}", e);
                    return Err(ErrorData::internal_error(
                        format!("Search failed: {}. Please check that Qdrant is running and accessible.", e),
                        None
                    ));
                }
            }
        };
        
        let content = Content::json(result)
            .map_err(|e| ErrorData::internal_error(format!("Failed to create JSON content: {}", e), None))?;
        Ok(CallToolResult::success(vec![content]))
    }
    
    /// Store a memory with automatic embedding generation
    #[tool(description = "Store a memory with automatic embedding generation and indexing in both Redis and Qdrant")]
    pub async fn rag_store(
        &self,
        _params: Parameters<RagStoreParams>,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        // TODO: Implement store logic
        let result = serde_json::json!({
            "status": "not_implemented",
            "message": "Store functionality coming soon"
        });
        
        let content = Content::json(result)
            .map_err(|e| ErrorData::internal_error(format!("Failed to create JSON content: {}", e), None))?;
        Ok(CallToolResult::success(vec![content]))
    }
}

#[tool_handler]
impl ServerHandler for UnifiedRagService {}