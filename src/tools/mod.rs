use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RagSearchParams {
    /// The search query
    pub query: String,
    
    /// Maximum number of results to return (default: 20)
    #[serde(default = "default_limit")]
    pub limit: usize,
    
    /// Similarity threshold for semantic search (0.0-1.0, default: 0.7)
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    
    /// Filter by category
    pub category_filter: Option<String>,
    
    /// Filter by tags
    pub tags_filter: Option<Vec<String>>,
    
    /// Filter by instance IDs
    pub instance_filter: Option<Vec<String>>,
    
    /// Use hybrid search (cache + semantic)
    #[serde(default = "default_hybrid")]
    pub hybrid_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RagStoreParams {
    /// The content to store
    pub content: String,
    
    /// Category for the memory
    pub category: Option<String>,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Importance score (1-10)
    pub importance: Option<i32>,
    
    /// Chain ID to link memories
    pub chain_id: Option<String>,
    
    /// Parent memory ID
    pub parent_id: Option<String>,
    
    /// Thinking framework used
    pub framework: Option<String>,
}

fn default_limit() -> usize { 20 }
fn default_threshold() -> f32 { 0.7 }
fn default_hybrid() -> bool { true }