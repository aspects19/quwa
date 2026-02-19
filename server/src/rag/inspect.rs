use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct InspectRequest {
    pub query: String,
    pub top_k: Option<usize>,
    pub min_similarity: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct InspectHit {
    pub text: String,
    pub similarity: f32,
    pub source_type: String,
    pub source_id: String,
    pub file_name: Option<String>,
    pub orpha_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectResponse {
    pub query: String,
    pub hits: Vec<InspectHit>,
}

pub async fn inspect_vectors(
    State(state): State<AppState>,
    Json(payload): Json<InspectRequest>,
) -> Json<InspectResponse> {
    let query = payload.query.trim().to_string();
    let top_k = payload.top_k.unwrap_or(5).max(1).min(25);
    let min_similarity = payload.min_similarity.unwrap_or(0.0);

    if query.is_empty() {
        return Json(InspectResponse { query, hits: vec![] });
    }

    let embedding = match state.embedding_service.embed_text(&query).await {
        Ok(embedding) => embedding,
        Err(e) => {
            tracing::error!("Vector inspect embedding error: {}", e);
            return Json(InspectResponse { query, hits: vec![] });
        }
    };

    let results = match state.vector_store.search(embedding, top_k).await {
        Ok(results) => results,
        Err(e) => {
            tracing::error!("Vector inspect search error: {}", e);
            vec![]
        }
    };

    let hits = results
        .into_iter()
        .filter(|(_, similarity, _)| *similarity >= min_similarity)
        .map(|(text, similarity, metadata)| InspectHit {
            text,
            similarity,
            source_type: metadata.source_type,
            source_id: metadata.source_id,
            file_name: metadata.file_name,
            orpha_code: metadata.orpha_code,
        })
        .collect();

    Json(InspectResponse { query, hits })
}
