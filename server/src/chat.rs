use axum::{
    extract::{Json, State},
    response::sse::{Event, Sse},
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use rig::completion::Prompt;
use rig::client::{CompletionClient};

use crate::{AppState, auth::AppwriteClaims, rag::context_strategy};


#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ThinkingData {
    pub step: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseData {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct SourceData {
    pub source_type: String,
    pub source_id: String,
    pub relevance: f32,
}

pub async fn chat_handler(
    State(state): State<AppState>,
    claims: AppwriteClaims,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_message = payload.message.clone();
    
    // Clone state components for async task
    let vector_store = state.vector_store.clone();
    let db_pool = state.db_pool.clone();
    let gemini_client = state.gemini_client.clone();
    let embedding_service = state.embedding_service.clone();
    let user_id = claims.user_id.clone();
    let request_counter = state.request_counter.clone();
    
    // Create events stream with RAG integration
    let stream = async_stream::stream! {
        // Step 1: Analyzing query
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Analyzing patient query and symptoms...".to_string()
            })
            .unwrap());
        
        // Step 2: Generate embedding for query (optional - can be disabled to avoid rate limits)
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Generating semantic representation...".to_string()
            })
            .unwrap());
        
        
        // Check if embeddings are enabled (set ENABLE_EMBEDDINGS=false to skip)
        let enable_embeddings = std::env::var("ENABLE_EMBEDDINGS")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";
        
        let query_embedding = if enable_embeddings {
            // Use local embedding service (fast, no API calls)
            match embedding_service.embed_text(&user_message).await {
                Ok(emb) => emb,
                Err(e) => {
                    tracing::error!("Local embedding generation failed: {}", e);
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("thinking")
                        .json_data(ThinkingData {
                            step: "⚠️ Embedding failed - proceeding without context search...".to_string()
                        })
                        .unwrap());
                    vec![0.0; 384] // Fallback to zero vector
                }
            }
        } else {
            // Embeddings disabled - skip entirely
            tracing::info!("Embeddings disabled via ENABLE_EMBEDDINGS=false");
            vec![0.0; 384]
        };
        
        // Step 3: Retrieve user's uploaded files
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Searching medical knowledge base and patient records...".to_string()
            })
            .unwrap());
        
        // Get user's files for context strategy
        let user_files = match crate::db::queries::get_user_files(&db_pool, 
            crate::db::queries::get_or_create_user(&db_pool, &user_id, None, None)
                .await
                .map(|u| u.id)
                .unwrap_or(0)
        ).await {
            Ok(files) => files,
            Err(_) => vec![]
        };
        
        // Step 4: RAG vector search
        let rag_results = match vector_store.search(query_embedding, 5).await {
            Ok(results) => results,
            Err(e) => {
                tracing::error!("Vector search failed: {}", e);
                vec![]
            }
        };
        
        // Determine context strategy
        let _strategy = context_strategy::determine_strategy(&user_files, &user_message);
        
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: format!("Found {} relevant medical references", rag_results.len())
            })
            .unwrap());
        
        // Step 5: Build context from RAG results
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Cross-referencing with rare disease database...".to_string()
            })
            .unwrap());
        
        let context = build_rag_context(&rag_results);
        
        // Step 6: Generate response
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Formulating diagnosis and recommendations...".to_string()
            })
            .unwrap());
        
        // Build enhanced prompt with RAG context
        let enhanced_prompt = if !context.is_empty() {
            format!(
                "You are a medical AI assistant specializing in rare disease diagnosis.\n\n\
                 RELEVANT MEDICAL KNOWLEDGE:\n{}\n\n\
                 PATIENT QUERY: {}\n\n\
                 Based on the above medical knowledge and the patient's query, provide a detailed, \
                 evidence-based response. Include:\n\
                 1. Possible diagnoses or conditions\n\
                 2. Recommended diagnostic tests or evaluations\n\
                 3. Relevant symptoms to monitor\n\
                 4. When to seek immediate medical attention\n\n\
                 Be thorough but clear. Cite sources when referencing specific conditions.",
                context, user_message
            )
        } else {
            format!(
                "You are a medical AI assistant specializing in rare disease diagnosis.\n\n\
                 PATIENT QUERY: {}\n\n\
                 Provide a helpful medical response based on your knowledge. \
                 Note: No specific medical records or disease database entries were found for this query.",
                user_message
            )
        };
        
        // Add delay before chat generation API call to avoid rate limits
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        
        // Log chat generation request
        request_counter.log_chat_request(&format!("User query: {}", user_message.chars().take(50).collect::<String>()));
        
        // Call Gemini API for streaming response (with retry logic)
        const MAX_CHAT_RETRIES: u32 = 3;
        const CHAT_BASE_DELAY_MS: u64 = 2000;
        
        for attempt in 0..MAX_CHAT_RETRIES {
            match gemini_client
                .agent("gemini-2.0-flash-lite") 
                .preamble(&enhanced_prompt)
                .build()
                .prompt(&user_message)
                .await
            {
                Ok(response) => {
                    // Stream the complete response
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("response")
                        .json_data(ResponseData {
                            content: response
                        })
                        .unwrap());
                    break; // Success - exit retry loop
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    
                    // Check if it's a rate limit error
                    if error_msg.contains("429") || error_msg.contains("RESOURCE_EXHAUSTED") {
                        if attempt < MAX_CHAT_RETRIES - 1 {
                            let delay = CHAT_BASE_DELAY_MS * 2_u64.pow(attempt);
                            tracing::warn!(
                                "Chat API rate limit hit, retrying in {}ms (attempt {}/{})",
                                delay, attempt + 1, MAX_CHAT_RETRIES
                            );
                            
                            yield Ok::<Event, Infallible>(Event::default()
                                .event("thinking")
                                .json_data(ThinkingData {
                                    step: format!("⏳ Rate limit hit, waiting {}s before retry...", delay / 1000)
                                })
                                .unwrap());
                            
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                            continue;
                        }
                    }
                    
                    // Not a rate limit or out of retries - send error
                    tracing::error!("Gemini API error: {}", e);
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("response")
                        .json_data(ResponseData {
                            content: format!(
                                "I apologize, but I encountered an error: {}\n\n\
                                 This is likely due to API rate limits. Please wait a moment and try again.",
                                if error_msg.contains("429") { "Rate limit exceeded" } else { "API error" }
                            )
                        })
                        .unwrap());
                    break;
                }
            }
        }
        
        // Send sources if available
        if !rag_results.is_empty() {
            for (_text, score, metadata) in rag_results.iter().take(3) {
                yield Ok::<Event, Infallible>(Event::default()
                    .event("source")
                    .json_data(SourceData {
                        source_type: metadata.source_type.clone(),
                        source_id: metadata.source_id.clone(),
                        relevance: *score,
                    })
                    .unwrap());
            }
        }
        
        // Done - send completion event with data
        yield Ok::<Event, Infallible>(Event::default()
            .event("done")
            .json_data(serde_json::json!({"status": "complete"}))
            .unwrap());
    };

    Sse::new(stream)
}

// Build context string from RAG results
fn build_rag_context(results: &[(String, f32, crate::rag::vector_store::DocumentMetadata)]) -> String {
    if results.is_empty() {
        return String::new();
    }
    
    results
        .iter()
        .enumerate()
        .map(|(i, (text, score, metadata))| {
            let source = match metadata.source_type.as_str() {
                "orphadata" => format!("Orphadata: {}", metadata.source_id),
                "user_file" => format!("Patient File: {}", metadata.file_name.as_deref().unwrap_or("Unknown")),
                _ => format!("Source: {}", metadata.source_id),
            };
            format!("[{}] {} (relevance: {:.2})\n{}\n", i + 1, source, score, text)
        })
        .collect::<Vec<_>>()
        .join("\n---\n")
}