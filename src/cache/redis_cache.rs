use std::sync::Arc;
use async_trait::async_trait;
use deadpool_redis::Pool;
use redis::AsyncCommands;
use crate::cache::{CacheLayer, CacheStats};
use crate::error::Result;
use crate::models::{Memory, SearchRequest};
use md5;

#[derive(Clone)]
pub struct RedisCache {
    pool: Arc<Pool>,
    prefix: String,
}

impl RedisCache {
    pub fn new(pool: Arc<Pool>, instance_id: &str) -> Self {
        Self {
            pool,
            prefix: instance_id.to_string(),
        }
    }
    
    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
    
    fn make_thought_key(&self, thought_id: &str) -> String {
        format!("{}:Thoughts:{}", self.prefix, thought_id)
    }
    
    fn make_embedding_key(&self, content: &str) -> String {
        let hash = format!("{:x}", md5::compute(content));
        format!("um:embedding:{}", hash)
    }
    
    fn make_metadata_key(&self, thought_id: &str) -> String {
        format!("{}:thought_meta:{}", self.prefix, thought_id)
    }
    
    fn make_tag_key(&self, tag: &str) -> String {
        format!("{}:tags:{}", self.prefix, tag)
    }
    
    fn make_chain_key(&self, chain_id: &str) -> String {
        format!("{}:chains:{}", self.prefix, chain_id)
    }
    
    fn make_cache_key(&self, query_hash: &str) -> String {
        format!("um:cache:{}", query_hash)
    }
}

#[async_trait]
impl CacheLayer for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<Memory>> {
        let mut conn = self.pool.get().await?;
        let full_key = self.make_thought_key(key);
        
        // Try to get JSON data from Redis
        let data: Option<String> = conn.get(&full_key).await?;
        
        match data {
            Some(json) => {
                let memory: Memory = serde_json::from_str(&json)?;
                
                // Update metadata access count and last_accessed
                let metadata_key = self.make_metadata_key(key);
                let _ = conn.hincr::<_, _, _, i64>(
                    &metadata_key, 
                    "access_count", 
                    1
                ).await;
                let _ = conn.hset::<_, _, _, ()>(
                    &metadata_key,
                    "last_accessed",
                    chrono::Utc::now().to_rfc3339()
                ).await;
                
                Ok(Some(memory))
            }
            None => Ok(None)
        }
    }
    
    async fn set(&self, key: &str, memory: &Memory, ttl_seconds: Option<u64>) -> Result<()> {
        let mut conn = self.pool.get().await?;
        let thought_key = self.make_thought_key(key);
        let json = serde_json::to_string(memory)?;
        
        // Set the thought with optional TTL (though thoughts typically don't expire)
        if let Some(ttl) = ttl_seconds {
            conn.set_ex::<_, _, ()>(&thought_key, &json, ttl).await?;
        } else {
            conn.set::<_, _, ()>(&thought_key, &json).await?;
        }
        
        // Store metadata
        let metadata_key = self.make_metadata_key(key);
        let metadata = serde_json::json!({
            "thought_id": key,
            "instance": self.prefix,
            "importance": memory.metadata.importance,
            "category": memory.metadata.category,
            "tags": memory.metadata.tags,
            "created_at": memory.created_at.to_rfc3339(),
            "last_accessed": memory.created_at.to_rfc3339(),
            "access_count": 0
        });
        conn.set::<_, _, ()>(&metadata_key, metadata.to_string()).await?;
        
        // Index tags
        for tag in &memory.metadata.tags {
            let tag_key = self.make_tag_key(tag);
            conn.sadd::<_, _, ()>(&tag_key, key).await?;
        }
        
        // Add to chain if chain_id exists
        if let Some(chain_id) = &memory.metadata.chain_id {
            let chain_key = self.make_chain_key(chain_id);
            conn.rpush::<_, _, ()>(&chain_key, key).await?;
        }
        
        Ok(())
    }
    
    async fn search_cached(&self, request: &SearchRequest) -> Result<Vec<Memory>> {
        // Check if we have a cached search result first
        let mut conn = self.pool.get().await?;
        
        // Create query hash for cache lookup
        let query_hash = format!("{:x}", md5::compute(format!("{:?}", request)));
        let cache_key = self.make_cache_key(&query_hash);
        
        // Try to get cached results
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
            if let Ok(cached_result) = serde_json::from_str::<Vec<Memory>>(&cached) {
                return Ok(cached_result);
            }
        }
        
        // Otherwise, scan for thoughts
        let pattern = format!("{}:Thoughts:*", self.prefix);
        let mut cursor: u64 = 0;
        let mut results = Vec::new();
        
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;
            
            for key in keys {
                // Extract thought_id from key
                if let Some(thought_id) = key.strip_prefix(&format!("{}:Thoughts:", self.prefix)) {
                    if let Some(memory) = self.get(thought_id).await? {
                        // Apply filters
                        if let Some(ref category) = request.category_filter {
                            if memory.metadata.category.as_ref() != Some(category) {
                                continue;
                            }
                        }
                        
                        if let Some(ref tags_filter) = request.tags_filter {
                            let has_tag = tags_filter.iter().any(|tag| memory.metadata.tags.contains(tag));
                            if !has_tag {
                                continue;
                            }
                        }
                        
                        if let Some(ref instance_filter) = request.instance_filter {
                            if !instance_filter.contains(&memory.instance_id) {
                                continue;
                            }
                        }
                        
                        results.push(memory);
                        
                        if results.len() >= request.limit.unwrap_or(20) {
                            break;
                        }
                    }
                }
            }
            
            cursor = new_cursor;
            if cursor == 0 || results.len() >= request.limit.unwrap_or(20) {
                break;
            }
        }
        
        // Cache the results with TTL
        if !results.is_empty() {
            let _ = conn.set_ex::<_, _, ()>(
                &cache_key,
                serde_json::to_string(&results)?,
                3600 // 1 hour TTL
            ).await;
        }
        
        Ok(results)
    }
    
    async fn invalidate(&self, key: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;
        
        // Get the memory first to clean up related data
        if let Some(memory) = self.get(key).await? {
            // Remove from tags
            for tag in &memory.metadata.tags {
                let tag_key = self.make_tag_key(tag);
                conn.srem::<_, _, ()>(&tag_key, key).await?;
            }
            
            // Remove from chain
            if let Some(chain_id) = &memory.metadata.chain_id {
                let chain_key = self.make_chain_key(chain_id);
                conn.lrem::<_, _, ()>(&chain_key, 0, key).await?;
            }
        }
        
        // Delete the thought and metadata
        let thought_key = self.make_thought_key(key);
        let metadata_key = self.make_metadata_key(key);
        
        conn.del::<_, ()>(&thought_key).await?;
        conn.del::<_, ()>(&metadata_key).await?;
        
        Ok(())
    }
    
    async fn get_stats(&self) -> Result<CacheStats> {
        let mut conn = self.pool.get().await?;
        
        // Count thoughts using SCAN to avoid blocking
        let pattern = format!("{}:Thoughts:*", self.prefix);
        let mut cursor: u64 = 0;
        let mut total_keys = 0u64;
        
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await?;
            
            total_keys += keys.len() as u64;
            
            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }
        
        // Get memory info from Redis INFO
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await?;
        
        let mut memory_usage = 0u64;
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    memory_usage = value.parse().unwrap_or(0);
                    break;
                }
            }
        }
        
        Ok(CacheStats {
            total_keys,
            memory_usage_bytes: memory_usage,
            hit_rate: 0.0, // TODO: Implement hit rate tracking
            miss_rate: 0.0, // TODO: Implement miss rate tracking
            avg_retrieval_time_ms: 0.0, // TODO: Implement timing
        })
    }
}