use anyhow::{Result, Context};
use std::path::Path;

use crate::processing::{OrphanetProcessor};
use crate::embeddings::LocalEmbeddingService;
use crate::rag::vector_store::{RagVectorStore, DocumentMetadata};

/// Load Orphanet dataset into the vector store
pub async fn load_orphanet_data(
    vector_store: &RagVectorStore,
    embedding_service: &LocalEmbeddingService,
    dataset_path: &Path,
    limit: Option<usize>,
) -> Result<usize> {
    tracing::info!("Starting Orphanet dataset loading from {:?}", dataset_path);
    
    // Check if Orphanet data already exists in MongoDB
    let existing_count = vector_store.count_by_source("orphadata").await?;
    if existing_count > 0 {
        tracing::info!(
            "✓ Orphanet data already loaded ({} disorders), skipping re-processing",
            existing_count
        );
        return Ok(existing_count);
    }
    
    tracing::info!("No existing Orphanet data found, loading fresh...");
    
    // Parse XML
    let processor = OrphanetProcessor::new(limit);
    let disorders = processor.parse_xml(dataset_path)
        .context("Failed to parse Orphanet XML")?;
    
    tracing::info!("Parsed {} disorders, generating embeddings...", disorders.len());
    
    // Convert to embedable texts
    let _texts: Vec<String> = disorders
        .iter()
        .map(|d| d.to_embedable_text())
        .collect();
    
    // Generate embeddings in batches to avoid memory issues
    const BATCH_SIZE: usize = 50;
    let mut total_added = 0;
    
    for (batch_idx, chunk) in disorders.chunks(BATCH_SIZE).enumerate() {
        let batch_texts: Vec<String> = chunk
            .iter()
            .map(|d| d.to_embedable_text())
            .collect();
        
        tracing::info!(
            "Processing batch {}/{} ({} disorders)...",
            batch_idx + 1,
            (disorders.len() + BATCH_SIZE - 1) / BATCH_SIZE,
            batch_texts.len()
        );
        
        // Generate embeddings for this batch
        let embeddings = embedding_service.embed_batch(batch_texts.clone())
            .await
            .context("Failed to generate batch embeddings")?;
        
        // Add to vector store
        for (idx, (disorder, embedding)) in chunk.iter().zip(embeddings.iter()).enumerate() {
            let doc_id = format!("orphanet_{}", disorder.orpha_code);
            let metadata = DocumentMetadata {
                source_type: "orphadata".to_string(),
                source_id: disorder.orpha_code.clone(),
                file_name: None,
                orpha_code: Some(disorder.orpha_code.clone()),
            };
            
            vector_store.add_document(
                doc_id,
                batch_texts[idx].clone(),
                embedding.clone(),
                metadata,
            ).await?;
            
            total_added += 1;
        }
        
        tracing::info!(
            "Batch {} complete. Total processed: {}/{}",
            batch_idx + 1,
            total_added,
            disorders.len()
        );
    }
    
    tracing::info!(
        "✓ Successfully loaded {} Orphanet disorders into vector store",
        total_added
    );
    
    Ok(total_added)
}

/// Load Orphanet data with configuration from environment variables
pub async fn load_orphanet_from_env(
    vector_store: &RagVectorStore,
    embedding_service: &LocalEmbeddingService,
) -> Result<usize> {
    let dataset_path = std::env::var("ORPHANET_DATASET_PATH")
        .unwrap_or_else(|_| "dataset/en_product4.xml".to_string());
    
    let limit = std::env::var("ORPHANET_LIMIT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok());
    
    load_orphanet_data(
        vector_store,
        embedding_service,
        Path::new(&dataset_path),
        limit,
    ).await
}
