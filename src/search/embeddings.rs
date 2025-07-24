use async_openai::{Client, config::OpenAIConfig};
use async_openai::types::{CreateEmbeddingRequestArgs, EmbeddingInput};
use crate::error::{Result, UnifiedRagError};

pub struct EmbeddingGenerator {
    client: Client<OpenAIConfig>,
    model: String,
}

impl EmbeddingGenerator {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| UnifiedRagError::Configuration("OPENAI_API_KEY not set".to_string()))?;
        
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        
        Ok(Self {
            client,
            model: "text-embedding-3-small".to_string(),
        })
    }
    
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(EmbeddingInput::String(text.to_string()))
            .build()?;
        
        let response = self.client
            .embeddings()
            .create(request)
            .await?;
        
        let embedding = response
            .data
            .first()
            .ok_or_else(|| UnifiedRagError::SearchError("No embedding returned".to_string()))?
            .embedding
            .clone();
        
        Ok(embedding)
    }
    
    pub async fn generate_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let inputs: Vec<String> = texts
            .into_iter()
            .map(|t| t.to_string())
            .collect();
        
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(inputs)
            .build()?;
        
        let response = self.client
            .embeddings()
            .create(request)
            .await?;
        
        let embeddings = response
            .data
            .into_iter()
            .map(|e| e.embedding)
            .collect();
        
        Ok(embeddings)
    }
}