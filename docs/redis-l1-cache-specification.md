# Redis L1 Cache Structure Specification
## For unified-rag Implementation

### Overview
This document specifies the existing Redis L1 cache structure used by the LegacyMind system. The unified-rag implementation must be compatible with these established patterns to ensure seamless integration.

### Key Pattern Summary

| Component | Key Pattern | Example |
|-----------|------------|---------|
| Thoughts | `{instance}:Thoughts:{thought_id}` | `CC:Thoughts:123e4567-e89b-12d3-a456-426614174000` |
| Embeddings | `um:embedding:{md5_hash}` | `um:embedding:5d41402abc4b2a76b9719d911017c592` |
| Thought Metadata | `{instance}:thought_meta:{thought_id}` | `CC:thought_meta:123e4567-e89b-12d3-a456-426614174000` |
| Tags | `{instance}:tags:{tag}` | `CC:tags:architecture` |
| Chains | `{instance}:chains:{chain_id}` | `CC:chains:chain-123` |
| Chain Metadata | `Chains:metadata:{chain_id}` | `Chains:metadata:chain-123` |
| Boost Scores | `{instance}:boost_scores` | `CC:boost_scores` |
| Identity | `{instance}:identity` | `CC:identity` |
| Identity Documents | `{instance}:identity:{field_type}:{document_id}` | `CC:identity:traits:doc-456` |
| Cache | `um:cache:{query_hash}` | `um:cache:abc123def456` |

### Detailed Data Structures

#### 1. Thought Storage
**Key Pattern:** `{instance}:Thoughts:{thought_id}`  
**Storage Type:** JSON String  
**Data Structure:**
```json
{
  "id": "uuid-v4",
  "thought": "The actual thought content",
  "content": "The actual thought content (duplicate for search)",
  "timestamp": "2024-01-15T10:30:00Z",
  "instance": "CC",
  "chain_id": "optional-chain-id",
  "thought_number": 1,
  "total_thoughts": 10,
  "next_thought_needed": false,
  "similarity": null
}
```

#### 2. Embedding Storage
**Key Pattern:** `um:embedding:{md5_hash}`  
**Storage Type:** JSON Array of floats  
**Hash Calculation:** MD5 hash of the text content  
**Data Structure:**
```json
[0.1234, -0.5678, 0.9012, ...] // 1536-dimensional vector for OpenAI embeddings
```
**TTL:** Configurable, typically 86400 seconds (24 hours)

#### 3. Thought Metadata
**Key Pattern:** `{instance}:thought_meta:{thought_id}`  
**Storage Type:** JSON  
**Data Structure:**
```json
{
  "thought_id": "uuid-v4",
  "instance": "CC",
  "importance": 8,
  "relevance": 7,
  "category": "technical",
  "tags": ["redis", "architecture", "cache"],
  "created_at": "2024-01-15T10:30:00Z",
  "last_accessed": "2024-01-15T11:00:00Z",
  "access_count": 5
}
```

#### 4. Tag Indexing
**Key Pattern:** `{instance}:tags:{tag}`  
**Storage Type:** Redis Set  
**Members:** Thought IDs that have this tag  
**Example:** `SMEMBERS CC:tags:architecture` returns `["thought-id-1", "thought-id-2"]`

#### 5. Chain Storage
**Key Pattern:** `{instance}:chains:{chain_id}`  
**Storage Type:** Redis List  
**Members:** Thought IDs in the chain, ordered by thought_number

#### 6. Chain Metadata
**Key Pattern:** `Chains:metadata:{chain_id}`  
**Storage Type:** JSON  
**Data Structure:**
```json
{
  "chain_id": "chain-unique-id",
  "instance": "CC",
  "thought_count": 10,
  "created_at": "2024-01-15T10:30:00Z",
  "last_updated": "2024-01-15T11:00:00Z"
}
```

#### 7. Boost Scores
**Key Pattern:** `{instance}:boost_scores`  
**Storage Type:** Redis Sorted Set  
**Members:** Thought IDs  
**Scores:** Boost values (floating point)  
**Operations:** `ZINCRBY` for updating, `ZSCORE` for retrieval

#### 8. Search Cache
**Key Pattern:** `um:cache:{query_hash}`  
**Storage Type:** JSON  
**Data Structure:**
```json
{
  "query": "original search query",
  "results": [...], // Array of search results
  "timestamp": "2024-01-15T10:30:00Z",
  "ttl": 3600
}
```

### Additional Redis Structures

#### Metrics and Analytics
- **Access Count:** `{instance}:metrics:access_count` (Time Series)
- **Thought Count:** `{instance}:metrics:thought_count` (Time Series)
- **Last Access:** `{thought_key}:last_access` (String timestamp)

#### Bloom Filters
- **Pattern:** `{instance}:bloom:thoughts`
- **Purpose:** Fast duplicate detection for thoughts

#### Event Streams
- **Pattern:** `{instance}:feedback_events`
- **Type:** Redis Streams (XADD)
- **Purpose:** Asynchronous event processing

### Critical Implementation Notes

1. **Atomic Operations**: The system uses Lua scripts for atomic operations when storing thoughts to ensure consistency across multiple keys.

2. **TTL Management**: 
   - Embeddings: 24 hours default
   - Cache entries: Configurable, typically 1 hour
   - Thought data: No expiration
   - Metrics: 30 days

3. **JSON Storage**: All complex objects are stored as JSON strings, not Redis JSON documents (for compatibility).

4. **Instance Isolation**: Each instance (CC, CCB, Trips, etc.) has its own namespace to prevent data collision.

5. **Search Fallback**: When Redis Search is unavailable, the system falls back to SCAN operations with pattern matching.

### Integration Requirements for unified-rag

1. **Respect Existing Keys**: Do not modify the key patterns or data structures.

2. **Instance Awareness**: The unified-rag must be instance-aware and use the appropriate prefix.

3. **Embedding Compatibility**: Store embeddings using the exact `um:embedding:{md5_hash}` pattern.

4. **Atomic Updates**: When updating related data (thought + metadata + indexes), use transactions or Lua scripts.

5. **Cache Strategy**: Implement a two-tier cache:
   - L1: Redis with the patterns above
   - L2: Qdrant for semantic search

6. **Error Handling**: Gracefully handle Redis connection failures with appropriate fallbacks.

### Example Redis Commands

```bash
# Get a thought
GET "CC:Thoughts:123e4567-e89b-12d3-a456-426614174000"

# Get embedding
GET "um:embedding:5d41402abc4b2a76b9719d911017c592"

# Get thoughts by tag
SMEMBERS "CC:tags:architecture"

# Get boost score
ZSCORE "CC:boost_scores" "thought-id-123"

# Scan for all thoughts in an instance
SCAN 0 MATCH "CC:Thoughts:*" COUNT 100
```

### Version Compatibility
This specification is based on the current implementation as of January 2025 and is compatible with:
- unified-intelligence v1.0
- unified-mind v1.0
- Redis 7.0+ with RedisJSON and RediSearch modules