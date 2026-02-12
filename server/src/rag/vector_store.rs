use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::embeddings::LocalEmbeddingService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_type: String,
    pub source_id: String,
    pub file_name: Option<String>,
    pub orpha_code: Option<String>,
}

pub struct RagVectorStore {
    pool: PgPool,
    embedding_service: std::sync::Arc<LocalEmbeddingService>,
}

impl RagVectorStore {
    pub async fn new(
        pool: &PgPool,
        embedding_service: std::sync::Arc<LocalEmbeddingService>,
    ) -> Result<Self> {
        tracing::info!("Initialized PostgreSQL vector store with pgvector");
        
        Ok(Self {
            pool: pool.clone(),
            embedding_service,
        })
    }
    
    pub async fn add_document(
        &self,
        _id: String,
        text: String,
        embedding: Vec<f32>,
        metadata: DocumentMetadata,
    ) -> Result<()> {
        crate::db::queries::add_embedding(
            &self.pool,
            text,
            embedding,
            metadata.source_type,
            metadata.source_id,
            metadata.file_name,
            metadata.orpha_code,
        )
        .await
        .context("Failed to insert document into vector store")?;
        
        Ok(())
    }
    
    pub async fn search(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<(String, f32, DocumentMetadata)>> {
        let results = crate::db::queries::search_embeddings(
            &self.pool,
            query_embedding,
            top_k as i64,
        )
        .await?;
        
        // Convert database results to expected format
        let formatted_results = results
            .into_iter()
            .map(|(text, similarity, source_type, source_id, file_name, orpha_code)| {
                let metadata = DocumentMetadata {
                    source_type,
                    source_id,
                    file_name,
                    orpha_code,
                };
                (text, similarity, metadata)
            })
            .collect();
        
        Ok(formatted_results)
    }
    
    pub async fn count(&self) -> usize {
        crate::db::queries::count_embeddings(&self.pool)
            .await
            .unwrap_or(0) as usize
    }
    
    pub async fn count_by_source(&self, source_type: &str) -> Result<usize> {
        let count = crate::db::queries::count_embeddings_by_source(&self.pool, source_type)
            .await?;
        Ok(count as usize)
    }
}
