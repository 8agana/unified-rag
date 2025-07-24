# Redis Cache Migration Guide for unified-rag

## Current vs Required Implementation

### Current Implementation (unified-rag)
```rust
// Current key pattern
prefix: format!("rag:{}:", instance_id)
// Example: "rag:CC:memory-id"
```

### Required Implementation (LegacyMind Compatible)
The unified-rag needs to support two distinct cache patterns:

1. **Embedding Cache** (from unified-mind):
   ```rust
   // Key: um:embedding:{md5_hash}
   let key = format!("um:embedding:{:x}", md5::compute(text));
   ```

2. **Thought/Memory Cache** (from unified-intelligence):
   ```rust
   // Key: {instance}:Thoughts:{thought_id}
   let key = format!("{}:Thoughts:{}", instance_id, thought_id);
   ```

## Implementation Changes Required

### 1. Update RedisCache Key Generation
```rust
impl RedisCache {
    pub fn new(pool: Arc<Pool>, instance_id: &str) -> Self {
        Self {
            pool,
            instance_id: instance_id.to_string(), // Store instance_id
            // Remove prefix field - we'll use context-aware keys
        }
    }
    
    fn make_embedding_key(&self, text: &str) -> String {
        format!("um:embedding:{:x}", md5::compute(text))
    }
    
    fn make_thought_key(&self, thought_id: &str) -> String {
        format!("{}:Thoughts:{}", self.instance_id, thought_id)
    }
    
    fn make_metadata_key(&self, thought_id: &str) -> String {
        format!("{}:thought_meta:{}", self.instance_id, thought_id)
    }
    
    fn make_tag_key(&self, tag: &str) -> String {
        format!("{}:tags:{}", self.instance_id, tag)
    }
}
```

### 2. Store Memories as Thoughts
Update the Memory model to include thought-compatible fields:

```rust
// When storing a Memory, convert to thought format:
let thought_data = json!({
    "id": memory.id,
    "thought": memory.content,
    "content": memory.content, // Duplicate for search compatibility
    "timestamp": memory.created_at.to_rfc3339(),
    "instance": self.instance_id,
    "chain_id": memory.metadata.get("chain_id"),
    "thought_number": 1,
    "total_thoughts": 1,
    "next_thought_needed": false,
    "similarity": memory.similarity_score
});
```

### 3. Implement Embedding Cache
```rust
async fn cache_embedding(&self, text: &str, embedding: &[f32]) -> Result<()> {
    let mut conn = self.pool.get().await?;
    let key = self.make_embedding_key(text);
    let json = serde_json::to_string(embedding)?;
    
    // Store with 24-hour TTL
    conn.set_ex::<_, _, ()>(&key, json, 86400).await?;
    Ok(())
}

async fn get_cached_embedding(&self, text: &str) -> Result<Option<Vec<f32>>> {
    let mut conn = self.pool.get().await?;
    let key = self.make_embedding_key(text);
    
    let data: Option<String> = conn.get(&key).await?;
    match data {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None)
    }
}
```

### 4. Update Search Implementation
```rust
async fn search_cached(&self, request: &SearchRequest) -> Result<Vec<Memory>> {
    let mut conn = self.pool.get().await?;
    
    // Use the correct pattern for thoughts
    let pattern = format!("{}:Thoughts:*", self.instance_id);
    let keys: Vec<String> = conn.keys(pattern).await?;
    
    let mut results = Vec::new();
    for key in keys {
        if let Some(data) = conn.get::<_, Option<String>>(&key).await? {
            // Parse thought format and convert to Memory
            if let Ok(thought) = serde_json::from_str::<serde_json::Value>(&data) {
                let memory = Memory {
                    id: thought["id"].as_str().unwrap_or_default().to_string(),
                    content: thought["content"].as_str().unwrap_or_default().to_string(),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        thought["timestamp"].as_str().unwrap_or_default()
                    ).unwrap_or_else(|_| chrono::Utc::now()),
                    metadata: MemoryMetadata {
                        source: Some("thought".to_string()),
                        category: None,
                        tags: vec![],
                        instance: Some(self.instance_id.clone()),
                    },
                    similarity_score: thought["similarity"].as_f64().map(|f| f as f32),
                };
                
                // Apply filters...
                results.push(memory);
            }
        }
    }
    
    Ok(results)
}
```

### 5. Add Metadata Support
```rust
async fn store_metadata(&self, thought_id: &str, memory: &Memory) -> Result<()> {
    let mut conn = self.pool.get().await?;
    let key = self.make_metadata_key(thought_id);
    
    let metadata = json!({
        "thought_id": thought_id,
        "instance": self.instance_id,
        "importance": memory.metadata.importance.unwrap_or(5),
        "category": memory.metadata.category,
        "tags": memory.metadata.tags,
        "created_at": memory.created_at.to_rfc3339(),
    });
    
    conn.set::<_, _, ()>(&key, metadata.to_string()).await?;
    
    // Update tag indexes
    for tag in &memory.metadata.tags {
        let tag_key = self.make_tag_key(tag);
        conn.sadd::<_, _, ()>(&tag_key, thought_id).await?;
    }
    
    Ok(())
}
```

## Testing Compatibility

1. **Verify Key Patterns**:
   ```bash
   redis-cli KEYS "CC:Thoughts:*"
   redis-cli KEYS "um:embedding:*"
   ```

2. **Test Data Retrieval**:
   - unified-rag should be able to read thoughts created by unified-intelligence
   - unified-intelligence should be able to read data stored by unified-rag

3. **Cross-Tool Validation**:
   - Create a thought using ui_think
   - Retrieve it using unified-rag
   - Verify all fields are preserved

## Configuration Updates

Update the unified-rag configuration to support the new structure:

```toml
[cache.redis]
# Instance ID must match the federation member
instance_id = "CC"  # or "CCB", "Trips", etc.

# Embedding cache TTL (seconds)
embedding_ttl = 86400  # 24 hours

# Thought cache - no TTL
thought_ttl = 0  # No expiration
```

## Summary of Changes

1. **Remove** the `rag:` prefix from all keys
2. **Implement** separate key patterns for embeddings and thoughts
3. **Store** Memory objects in thought-compatible format
4. **Add** metadata storage with tag indexing
5. **Update** search to use the correct key patterns
6. **Ensure** instance isolation through proper prefixing

This migration ensures unified-rag integrates seamlessly with the existing LegacyMind Redis structure while maintaining backward compatibility.