use anyhow::{Result, Context};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Local embedding service using FastEmbed (all-MiniLM-L6-v2)
/// This allows fast, offline embeddings without API calls
pub struct LocalEmbeddingService {
    model: Arc<Mutex<TextEmbedding>>,
}

impl LocalEmbeddingService {
    /// Initialize the local embedding model
    /// Downloads model on first run (~50MB), then cached locally
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing local embedding model (all-MiniLM-L6-v2)...");
        
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
        ).context("Failed to initialize local embedding model")?;
        
        tracing::info!("Local embedding model loaded successfully");
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
    
    /// Generate embedding for a single text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let model = self.model.lock().await;
        
        let embeddings = model
            .embed(vec![text.to_string()], None)
            .context("Failed to generate embedding")?;
        
        let embedding = embeddings
            .first()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?;
        
        Ok(embedding.clone())
    }
    
    /// Generate embeddings for multiple texts (batch processing)
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        
        let model = self.model.lock().await;
        
        tracing::debug!("Generating {} embeddings in batch", texts.len());
        
        let embeddings = model
            .embed(texts, None)
            .context("Failed to generate batch embeddings")?;
        
        Ok(embeddings)
    }
    
    /// Get the embedding dimension (384 for all-MiniLM-L6-v2)
    pub fn dimension(&self) -> usize {
        384
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embed_text() {
        let service = LocalEmbeddingService::new().unwrap();
        let embedding = service.embed_text("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 384);
    }
    
    #[tokio::test]
    async fn test_embed_batch() {
        let service = LocalEmbeddingService::new().unwrap();
        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
        ];
        let embeddings = service.embed_batch(texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }
}
