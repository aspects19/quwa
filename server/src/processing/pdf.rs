use anyhow::{Result, Context};
use bytes::Bytes;
use rig::providers::gemini;
use rig::client::EmbeddingsClient;

pub struct PdfProcessor {
    gemini_client: gemini::Client,
}

impl PdfProcessor {
    pub fn new(api_key: &str) -> Result<Self> {
        let gemini_client = gemini::Client::new(api_key)?;
        Ok(Self { gemini_client })
    }
    
    pub async fn process_pdf(&self, file_data: Bytes, counter: Option<&crate::request_counter::RequestCounter>) -> Result<Vec<(String, Vec<f32>)>> {
        // Extract text from PDF
        let text = self.extract_text(file_data)?;
        
        // Chunk text
        let chunks = self.chunk_text(&text)?;
        
        // Generate embeddings
        let embeddings = self.generate_embeddings(chunks, counter).await?;
        
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
    
    async fn generate_embeddings(&self, chunks: Vec<String>, counter: Option<&crate::request_counter::RequestCounter>) -> Result<Vec<(String, Vec<f32>)>> {
        let mut results = Vec::new();
        
        // Process chunks with progress logging
        for (idx, chunk) in chunks.iter().enumerate() {
            tracing::debug!("Generating embedding for chunk {}/{}", idx + 1, chunks.len());
            
            let context = format!("PDF chunk {}/{}", idx + 1, chunks.len());
            let embedding = self.generate_single_embedding(chunk, counter, &context).await?;
            results.push((chunk.clone(), embedding));
            
            // Add delay to avoid rate limiting (5 seconds between requests)
            // This keeps us under Gemini's free tier limit of ~15 requests/minute
            if idx < chunks.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
            }
        }
        
        Ok(results)
    }
    
    async fn generate_single_embedding(&self, text: &str, counter: Option<&crate::request_counter::RequestCounter>, context: &str) -> Result<Vec<f32>> {
        // Log request if counter is provided
        if let Some(c) = counter {
            c.log_embedding_request(context);
        }
        
        // Use Gemini's text-embedding-004 model
        let embeddings = self.gemini_client
            .embeddings("text-embedding-004")
            .document(text)
            .context("Failed to create embedding document")?
            .build()
            .await
            .context("Failed to generate embedding")?;
        
        // Extract the first embedding from the result (tuple of (text, embeddings))
        let (_, embedding_data) = embeddings
            .first()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?;
        
        // Get the vec from the embedding (convert from f64 to f32)
        let embedding_vec: Vec<f32> = embedding_data.first().vec.iter().map(|v| *v as f32).collect();
        
        Ok(embedding_vec)
    }
}
