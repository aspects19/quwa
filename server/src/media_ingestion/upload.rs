use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{AppState, auth::AppwriteClaims};
use super::validation::{validate_file, determine_file_type};

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub status: String,
}

pub async fn handle_file_upload(
    State(state): State<AppState>,
    claims: AppwriteClaims, // Extracted from JWT middleware
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    // Get or create user (now with email and name from Appwrite API)
    let user = crate::db::queries::get_or_create_user(
        &state.db_pool,
        &claims.user_id,
        claims.email.as_deref(),
        claims.name.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut file_data = None;
    let mut file_name = String::new();
    let mut content_type = String::new();
    
    // Parse multipart data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            file_name = field.file_name().unwrap_or("unknown").to_string();
            content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            
            let data = field.bytes().await.map_err(|e| {
                (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e))
            })?;
            
            file_data = Some(data);
        }
    }
    
    let file_bytes = file_data.ok_or((
        StatusCode::BAD_REQUEST,
        "No file provided".to_string(),
    ))?;
    
    // Validate file
    validate_file(&file_name, &file_bytes).map_err(|e| {
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;
    
    let file_id = Uuid::new_v4();
    let file_type = determine_file_type(&content_type);
    
    // For now, skip Appwrite upload and store file temporarily
    // TODO: Integrate actual Appwrite storage
    let appwrite_file_id = file_id.to_string();
    let appwrite_bucket_id = "medical_files".to_string();
    
    // Save metadata to database
    let uploaded_file = crate::db::models::UploadedFile {
        id: file_id,
        user_id: user.id,
        file_name: file_name.clone(),
        file_type: file_type.clone(),
        mime_type: Some(content_type),
        file_size_bytes: Some(file_bytes.len() as i64),
        appwrite_file_id,
        appwrite_bucket_id,
        processing_status: "pending".to_string(),
        upload_date: chrono::Utc::now(),
        processed_at: None,
        error_message: None,
    };
    
    crate::db::queries::create_uploaded_file(&state.db_pool, &uploaded_file)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Trigger async processing
    let state_clone = state.clone();
    let db_pool_clone = state.db_pool.clone();
    tokio::spawn(async move {
        if let Err(e) = process_uploaded_file(state_clone, file_id, file_type, file_bytes).await {
            tracing::error!("File processing failed for {}: {}", file_id, e);
            
            // Update status to failed
            let _ = crate::db::queries::update_file_status(
                &db_pool_clone,
                file_id,
                "failed",
                Some(&e.to_string()),
            ).await;
        }
    });
    
    Ok(Json(UploadResponse {
        file_id: file_id.to_string(),
        file_name,
        status: "processing".to_string(),
    }))
}

async fn process_uploaded_file(
    state: AppState,
    file_id: Uuid,
    file_type: String,
    file_data: bytes::Bytes,
) -> anyhow::Result<()> {
    tracing::info!("Processing file {} of type {}", file_id, file_type);
    
    // Update status to processing
    crate::db::queries::update_file_status(&state.db_pool, file_id, "processing", None).await?;
    
    match file_type.as_str() {
        "pdf" => {
            // Process PDF
            let embeddings = state.pdf_processor.process_pdf(file_data, Some(&state.request_counter)).await?;
            
            // Store in vector store
            for (idx, (text, embedding)) in embeddings.iter().enumerate() {
                let embedding_id = format!("{}_{}", file_id, idx);
                
                state.vector_store.add_document(
                    embedding_id.clone(),
                    text.clone(),
                    embedding.clone(),
                    crate::rag::vector_store::DocumentMetadata {
                        source_type: "user_file".to_string(),
                        source_id: file_id.to_string(),
                        file_name: None,
                        orpha_code: None,
                    },
                ).await?;
                
                // Store metadata in database
                crate::db::queries::create_embedding_metadata(
                    &state.db_pool,
                    file_id,
                    idx as i32,
                    text,
                    &embedding_id,
                ).await?;
            }
            
            tracing::info!("PDF processing completed: {} chunks", embeddings.len());
        }
        "image" => {
            // Process image
            let (description, embedding) = state.image_processor.process_image(file_data).await?;
            
            let embedding_id = format!("{}_0", file_id);
            
            // Store in vector store
            state.vector_store.add_document(
                embedding_id.clone(),
                description.clone(),
                embedding,
                crate::rag::vector_store::DocumentMetadata {
                    source_type: "user_file".to_string(),
                    source_id: file_id.to_string(),
                    file_name: None,
                    orpha_code: None,
                },
            ).await?;
            
            // Store metadata in database
            crate::db::queries::create_embedding_metadata(
                &state.db_pool,
                file_id,
                0,
                &description,
                &embedding_id,
            ).await?;
            
            tracing::info!("Image processing completed");
        }
        _ => {
            anyhow::bail!("Unsupported file type: {}", file_type);
        }
    }
    
    // Update status to completed
    crate::db::queries::update_file_status(&state.db_pool, file_id, "completed", None).await?;
    
    Ok(())
}
