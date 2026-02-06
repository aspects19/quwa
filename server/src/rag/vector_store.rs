use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_type: String, // "user_file" or "orphadata"
    pub source_id: String,
    pub file_name: Option<String>,
    pub orpha_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub metadata: DocumentMetadata,
}

pub struct RagVectorStore {
    documents: Arc<RwLock<Vec<Document>>>,
}

impl RagVectorStore {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn add_document(
        &self,
        id: String,
        text: String,
        embedding: Vec<f32>,
        metadata: DocumentMetadata,
    ) -> Result<()> {
        let mut docs = self.documents.write().await;
        docs.push(Document {
            id,
            text,
            embedding,
            metadata,
        });
        Ok(())
    }
    
    pub async fn search(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<(String, f32, DocumentMetadata)>> {
        let docs = self.documents.read().await;
        
        let mut results: Vec<_> = docs
            .iter()
            .map(|doc| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                (doc.text.clone(), similarity, doc.metadata.clone())
            })
            .collect();
        
        // Sort by similarity (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top_k
        results.truncate(top_k);
        
        Ok(results)
    }
    
    pub async fn search_by_source(
        &self,
        query_embedding: Vec<f32>,
        source_type: &str,
        top_k: usize,
    ) -> Result<Vec<(String, f32, DocumentMetadata)>> {
        let docs = self.documents.read().await;
        
        let mut results: Vec<_> = docs
            .iter()
            .filter(|doc| doc.metadata.source_type == source_type)
            .map(|doc| {
                let similarity = cosine_similarity(&query_embedding, &doc.embedding);
                (doc.text.clone(), similarity, doc.metadata.clone())
            })
            .collect();
        
        // Sort by similarity (highest first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top_k
        results.truncate(top_k);
        
        Ok(results)
    }
    
    pub async fn count(&self) -> usize {
        let docs = self.documents.read().await;
        docs.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_search() {
        let store = RagVectorStore::new();
        
        let metadata = DocumentMetadata {
            source_type: "test".to_string(),
            source_id: "1".to_string(),
            file_name: None,
            orpha_code: None,
        };
        
        store.add_document(
            "doc1".to_string(),
            "test document".to_string(),
            vec![0.1, 0.2, 0.3],
            metadata,
        ).await.unwrap();
        
        assert_eq!(store.count().await, 1);
        
        let results = store.search(vec![0.1, 0.2, 0.3], 1).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
