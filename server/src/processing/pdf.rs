use anyhow::{Result, Context};
use bytes::Bytes;
use std::sync::Arc;
use crate::embeddings::LocalEmbeddingService;

pub struct PdfProcessor {
    embedding_service: Arc<LocalEmbeddingService>,
}

impl PdfProcessor {
    pub fn new(embedding_service: Arc<LocalEmbeddingService>) -> Result<Self> {
        Ok(Self { embedding_service })
    }
    
    pub async fn process_pdf(&self, file_data: Bytes, _counter: Option<&crate::request_counter::RequestCounter>) -> Result<Vec<(String, Vec<f32>)>> {
        // Extract text from PDF
        let text = self.extract_text(file_data)?;
        
        // Chunk text
        let chunks = self.chunk_text(&text)?;
        
        // Generate embeddings using local FastEmbed
        let embeddings = self.generate_embeddings(chunks).await?;
        
        Ok(embeddings)
    }
    
    fn extract_text(&self, file_data: Bytes) -> Result<String> {
        use lopdf::Document;
        
        let doc = Document::load_mem(&file_data)
            .context("Failed to load PDF document")?;
        
        let mut text = String::new();
        let pages = doc.get_pages();
        
        for page_num in 1..=pages.len() {
            if let Ok(page_text) = doc.extract_text(&[page_num as u32]) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }
        
        if text.trim().is_empty() {
            anyhow::bail!("No text extracted from PDF");
        }
        
        Ok(text)
    }
    
    fn chunk_text(&self, text: &str) -> Result<Vec<String>> {
        // Simple chunking by size with overlap
        const CHUNK_SIZE: usize = 1500; // characters
        const OVERLAP: usize = 200;
        
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;
        
        while start < chars.len() {
            let end = (start + CHUNK_SIZE).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            
            if !chunk.trim().is_empty() {
                chunks.push(chunk.trim().to_string());
            }
            
            if end >= chars.len() {
                break;
            }
            
            start += CHUNK_SIZE - OVERLAP;
        }
        
        if chunks.is_empty() {
            anyhow::bail!("No chunks created from text");
        }
        
        Ok(chunks)
    }
    
    async fn generate_embeddings(&self, chunks: Vec<String>) -> Result<Vec<(String, Vec<f32>)>> {
        tracing::info!("Generating embeddings for {} PDF chunks using local FastEmbed", chunks.len());
        
        // Use local embedding service (batch processing, fast!)
        let embeddings = self.embedding_service.embed_batch(chunks.clone()).await?;
        
        // Combine chunks with their embeddings
        let results: Vec<(String, Vec<f32>)> = chunks.into_iter()
            .zip(embeddings.into_iter())
            .collect();
        
        Ok(results)
    }
}
