use std::sync::Arc;
use async_trait::async_trait;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, VectorParamsBuilder,
    PointStruct, SearchPointsBuilder, DeletePointsBuilder,
    Filter, Condition, UpsertPointsBuilder, GetPointsBuilder,
    PointId,
};
use qdrant_client::Payload;
use uuid::Uuid;

use crate::search::{SearchLayer, embeddings::EmbeddingGenerator};
use crate::error::{Result, UnifiedRagError};
use crate::models::{Memory, SearchRequest, SearchResult};

#[derive(Clone)]
pub struct QdrantSearch {
    client: Arc<Qdrant>,
    collection_name: String,
    embedding_generator: Arc<EmbeddingGenerator>,
}

impl QdrantSearch {
    pub async fn new(
        client: Arc<Qdrant>, 
        collection_name: String,
        embedding_generator: Arc<EmbeddingGenerator>
    ) -> Result<Self> {
        // Try to list collections with better error handling
        let collections = match client.list_collections().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to list Qdrant collections: {}. This might indicate Qdrant is not running or not accessible at the configured URL.", e);
                return Err(UnifiedRagError::Qdrant(format!(
                    "Failed to connect to Qdrant: {}. Please ensure Qdrant is running and accessible.", e
                )));
            }
        };
        
        let collection_exists = collections
            .collections
            .iter()
            .any(|c| c.name == collection_name);
        
        if !collection_exists {
            // Create collection with vector configuration
            match client.create_collection(
                CreateCollectionBuilder::new(&collection_name)
                    .vectors_config(VectorParamsBuilder::new(1536, Distance::Cosine))
            ).await {
                Ok(_) => {
                    tracing::info!("Created Qdrant collection: {}", collection_name);
                }
                Err(e) => {
                    tracing::error!("Failed to create Qdrant collection '{}': {}", collection_name, e);
                    return Err(UnifiedRagError::Qdrant(format!(
                        "Failed to create collection '{}': {}", collection_name, e
                    )));
                }
            }
        } else {
            tracing::info!("Using existing Qdrant collection: {}", collection_name);
        }
        
        Ok(Self {
            client,
            collection_name,
            embedding_generator,
        })
    }
}

#[async_trait]
impl SearchLayer for QdrantSearch {
    async fn search(&self, request: &SearchRequest) -> Result<SearchResult> {
        let start_time = std::time::Instant::now();
        
        // Generate embedding for query
        let query_embedding = self.embedding_generator
            .generate_embedding(&request.query)
            .await?;
        
        // Build search query
        let mut search_builder = SearchPointsBuilder::new(
            &self.collection_name,
            query_embedding.clone(),
            request.limit.unwrap_or(20) as u64,
        )
        .with_payload(true);
        
        // Add filters if specified
        let mut filter_conditions = vec![];
        
        if let Some(ref category) = request.category_filter {
            filter_conditions.push(Condition::matches("metadata.category", category.clone()));
        }
        
        if let Some(ref tags_filter) = request.tags_filter {
            for tag in tags_filter {
                filter_conditions.push(Condition::matches("metadata.tags", tag.clone()));
            }
        }
        
        if !filter_conditions.is_empty() {
            search_builder = search_builder.filter(Filter::must(filter_conditions));
        }
        
        // Execute search
        let search_results = self.client
            .search_points(search_builder)
            .await
            .map_err(|e| UnifiedRagError::Qdrant(e.to_string()))?;
        
        // Convert results to Memory objects
        let mut memories = Vec::new();
        for point in search_results.result {
            // Deserialize payload to Memory
            let memory_json = serde_json::to_value(&point.payload)?;
            let memory: Memory = serde_json::from_value(memory_json)?;
            memories.push(memory);
        }
        
        let search_time_ms = start_time.elapsed().as_millis() as u64;
        
        let total_results = memories.len();
        
        Ok(SearchResult {
            memories,
            search_id: Uuid::new_v4(),
            query_embedding: Some(query_embedding),
            cache_hits: 0, // Qdrant doesn't track cache hits
            total_results,
            search_time_ms,
        })
    }
    
    async fn index(&self, memory: &Memory) -> Result<()> {
        // Generate embedding if not present
        let embedding = match &memory.embedding {
            Some(e) => e.clone(),
            None => self.embedding_generator
                .generate_embedding(&memory.content)
                .await?
        };
        
        // Create payload from memory
        let payload_json = serde_json::to_value(memory)?;
        let payload: Payload = serde_json::from_value(payload_json)?;
        
        // Create point for Qdrant
        let point = PointStruct::new(
            memory.id.to_string(),
            embedding,
            payload
        );
        
        // Upsert point
        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![point]))
            .await
            .map_err(|e| UnifiedRagError::Qdrant(e.to_string()))?;
        
        Ok(())
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection_name)
                    .points(vec![id.to_string()])
            )
            .await
            .map_err(|e| UnifiedRagError::Qdrant(e.to_string()))?;
        
        Ok(())
    }
    
    async fn update_embedding(&self, id: &str, embedding: Vec<f32>) -> Result<()> {
        // Qdrant requires re-indexing the entire point to update embedding
        // First, get the existing point
        let existing_points = self.client
            .get_points(
                GetPointsBuilder::new(&self.collection_name, vec![PointId::from(id)])
            )
            .await
            .map_err(|e| UnifiedRagError::Qdrant(e.to_string()))?;
        
        if let Some(point) = existing_points.result.first() {
            // Create updated point with new embedding
            let payload: Payload = point.payload.clone().into();
            let updated_point = PointStruct::new(
                id.to_string(),
                embedding,
                payload
            );
            
            // Upsert the updated point
            self.client
                .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![updated_point]))
                .await
                .map_err(|e| UnifiedRagError::Qdrant(e.to_string()))?;
        }
        
        Ok(())
    }
}