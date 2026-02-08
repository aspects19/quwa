use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use mongodb::{Database, Collection, bson::doc};
use crate::embeddings::LocalEmbeddingService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub text: String,
    pub embedding: Vec<f32>,
    pub source_type: String,
    pub source_id: String,
    pub file_name: Option<String>,
    pub orpha_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_type: String,
    pub source_id: String,
    pub file_name: Option<String>,
    pub orpha_code: Option<String>,
}

pub struct RagVectorStore {
    collection: Collection<VectorDocument>,
    embedding_service: std::sync::Arc<LocalEmbeddingService>,
}

impl RagVectorStore {
    pub async fn new(
        db: &Database,
        embedding_service: std::sync::Arc<LocalEmbeddingService>,
    ) -> Result<Self> {
        let collection = db.collection::<VectorDocument>("embeddings");
        
        // Create vector search index if it doesn't exist
        // Note: For MongoDB Atlas, you need to create the search index manually in the UI
        // For local MongoDB, vector search may not be available
        
        tracing::info!("Initialized MongoDB vector store");
        
        Ok(Self {
            collection,
            embedding_service,
        })
    }
    
    pub async fn add_document(
        &self,
        id: String,
        text: String,
        embedding: Vec<f32>,
        metadata: DocumentMetadata,
    ) -> Result<()> {
        let doc = VectorDocument {
            id: None,
            text,
            embedding,
            source_type: metadata.source_type,
            source_id: metadata.source_id,
            file_name: metadata.file_name,
            orpha_code: metadata.orpha_code,
        };
        
        self.collection.insert_one(doc).await
            .context("Failed to insert document into vector store")?;
        
        Ok(())
    }
    
    pub async fn search(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<(String, f32, DocumentMetadata)>> {
        // Get all documents (for local MongoDB without vector search)
        let mut cursor = self.collection.find(doc! {}).await?;
        
        let mut documents = Vec::new();
        while cursor.advance().await? {
            documents.push(cursor.deserialize_current()?);
        }
        
        // Calculate cosine similarity for each document
        let mut results: Vec<_> = documents
            .into_iter()
            .map(|doc| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                let metadata = DocumentMetadata {
                    source_type: doc.source_type,
                    source_id: doc.source_id,
                    file_name: doc.file_name,
                    orpha_code: doc.orpha_code,
                };
                (doc.text, similarity, metadata)
            })
            .collect();
        
        // Sort by similarity (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top_k
        results.truncate(top_k);
        
        Ok(results)
    }
    
    pub async fn count(&self) -> usize {
        self.collection.count_documents(doc! {}).await.unwrap_or(0) as usize
    }
    
    pub async fn count_by_source(&self, source_type: &str) -> Result<usize> {
        let count = self.collection
            .count_documents(doc! { "source_type": source_type })
            .await?;
        Ok(count as usize)
    }
}

// Cosine similarity calculation
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (magnitude_a * magnitude_b)
}
