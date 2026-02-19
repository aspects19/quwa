use axum::{
    extract::{Json, State},
    response::sse::{Event, Sse},
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

use crate::{AppState, auth::AppwriteClaims};

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerateRequest {
    contents: Vec<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct GeminiGenerateResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

// ── Pipeline output types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ThinkingOnlyOutput {
    thinking_steps: Vec<String>,
}

/// Pass 1 output: AI-normalized clinical query
#[derive(Debug, Deserialize)]
struct NormalizedQuery {
    clinical_query: String,
    key_symptoms: Vec<String>,
}

/// Pass 2 output: AI-selected best candidate from the shortlist
#[derive(Debug, Deserialize)]
struct CandidateSelection {
    /// 0-based index into the top-K results list
    selected_index: usize,
    reasoning: String,
}

// ── Main handler ─────────────────────────────────────────────────────────────

pub async fn chat_handler(
    State(state): State<AppState>,
    claims: AppwriteClaims,
    Json(payload): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let user_message = payload.message.clone();

    let vector_store      = state.vector_store.clone();
    let db_pool           = state.db_pool.clone();
    let gemini_http_client = state.gemini_http_client.clone();
    let gemini_api_key    = state.gemini_api_key.clone();
    let gemini_model      = state.gemini_model.clone();
    let embedding_service = state.embedding_service.clone();
    let user_id           = claims.user_id.clone();
    let request_counter   = state.request_counter.clone();

    let stream = async_stream::stream! {

        // ── Step 0: acknowledge ──────────────────────────────────────────────
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData { step: "Analyzing symptoms...".to_string() })
            .unwrap());

        // ── Pass 1: AI symptom normalization ─────────────────────────────────
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData { step: "Normalizing to clinical terminology...".to_string() })
            .unwrap());

        let normalized = match call_gemini_normalize(
            &gemini_http_client,
            &gemini_api_key,
            &gemini_model,
            &user_message,
        ).await {
            Ok(n) => {
                if !n.key_symptoms.is_empty() {
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("thinking")
                        .json_data(ThinkingData {
                            step: format!("Key symptoms: {}", n.key_symptoms.join(", "))
                        })
                        .unwrap());
                }
                n
            }
            Err(e) => {
                tracing::warn!("Symptom normalization failed, using raw message: {}", e);
                NormalizedQuery {
                    clinical_query: user_message.clone(),
                    key_symptoms: vec![],
                }
            }
        };

        // ── Embed the normalized clinical query ──────────────────────────────
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData { step: "Generating semantic embedding...".to_string() })
            .unwrap());

        let enable_embeddings = std::env::var("ENABLE_EMBEDDINGS")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        let query_embedding = if enable_embeddings {
            match embedding_service.embed_text(&normalized.clinical_query).await {
                Ok(emb) => emb,
                Err(e) => {
                    tracing::error!("Embedding failed: {}", e);
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("thinking")
                        .json_data(ThinkingData {
                            step: "Embedding failed, proceeding without vector context...".to_string()
                        })
                        .unwrap());
                    vec![0.0; 384]
                }
            }
        } else {
            vec![0.0; 384]
        };

        // ── Vector search: top 10 candidates ─────────────────────────────────
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData { step: "Searching medical knowledge base...".to_string() })
            .unwrap());

        let _user_files = match crate::db::queries::get_user_files(
            &db_pool,
            crate::db::queries::get_or_create_user(&db_pool, &user_id, None, None)
                .await
                .map(|u| u.id)
                .unwrap_or(0)
        ).await {
            Ok(files) => files,
            Err(_) => vec![],
        };

        let rag_results = match vector_store.search(query_embedding, 10).await {
            Ok(results) => results,
            Err(e) => {
                tracing::error!("Vector search failed: {}", e);
                vec![]
            }
        };

        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: format!("Found {} candidate conditions, selecting best match...", rag_results.len())
            })
            .unwrap());

        // ── Pass 2: AI candidate selection ───────────────────────────────────
        let selected_match = if !rag_results.is_empty() {
            match call_gemini_select(
                &gemini_http_client,
                &gemini_api_key,
                &gemini_model,
                &user_message,
                &rag_results,
            ).await {
                Ok(sel) => {
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("thinking")
                        .json_data(ThinkingData {
                            step: format!("AI reasoning: {}", sel.reasoning)
                        })
                        .unwrap());
                    rag_results.get(sel.selected_index).cloned()
                        .or_else(|| rag_results.first().cloned())
                }
                Err(e) => {
                    tracing::warn!("Candidate selection failed, using top vector result: {}", e);
                    rag_results.first().cloned()
                }
            }
        } else {
            None
        };

        // ── Build enhanced prompt for final answer call ───────────────────────
        let context = build_rag_context(&rag_results);
        let selected_label = selected_match.as_ref().and_then(|(text, _score, _meta)| {
            parse_condition_from_text(text).map(|(name, _)| name)
        });

        let enhanced_prompt = match &selected_match {
            Some((text, score, meta)) => {
                let orpha = meta.orpha_code.as_deref().unwrap_or("unknown");
                let label = selected_label.as_deref().unwrap_or("unknown condition");
                format!(
                    "SELECTED BEST MATCH (AI-chosen from top 10 vector results):\n\
                     Condition: {} (Orpha: {})\nSimilarity: {:.2}\nContext:\n{}\n\n\
                     ALL CANDIDATES CONTEXT:\n{}\n\nPATIENT QUERY:\n{}\n\nKEY CLINICAL TERMS:\n{}",
                    label, orpha, score, text, context, user_message,
                    normalized.key_symptoms.join(", ")
                )
            }
            None => {
                format!(
                    "PATIENT QUERY:\n{}\n\nNo matching conditions found in the knowledge base.\n\
                     KEY CLINICAL TERMS:\n{}",
                    user_message,
                    normalized.key_symptoms.join(", ")
                )
            }
        };

        request_counter.log_chat_request(
            &format!("Gemini chat | User query: {}", user_message.chars().take(50).collect::<String>())
        );

        // ── Optional thinking steps (decorative) ─────────────────────────────
        const MAX_THINKING_RETRIES: u32 = 3;
        const THINKING_BASE_DELAY_MS: u64 = 1200;

        let mut thinking_steps: Vec<String> = Vec::new();
        for attempt in 0..MAX_THINKING_RETRIES {
            match call_gemini_thinking(
                &gemini_http_client,
                &gemini_api_key,
                &gemini_model,
                &enhanced_prompt,
                &user_message,
            ).await {
                Ok(output) => {
                    thinking_steps = output.thinking_steps;
                    break;
                }
                Err(e) => {
                    let err_str = e.to_string();
                    let rate_limited = err_str.contains("429")
                        || err_str.contains("rate_limit")
                        || err_str.contains("Rate limit");

                    if rate_limited && attempt < MAX_THINKING_RETRIES - 1 {
                        let jitter = ((attempt as u64 + 1) * 173) % 500;
                        let delay = THINKING_BASE_DELAY_MS * 2_u64.pow(attempt) + jitter;
                        tracing::warn!("Gemini thinking rate limited, retrying in {}ms", delay);
                        yield Ok::<Event, Infallible>(Event::default()
                            .event("thinking")
                            .json_data(ThinkingData {
                                step: format!("Rate limit hit, retrying in {}s...", delay / 1000)
                            })
                            .unwrap());
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        continue;
                    }
                    tracing::warn!("Could not generate thinking steps: {}", e);
                    break;
                }
            }
        }

        for step in thinking_steps.iter().take(6) {
            yield Ok::<Event, Infallible>(Event::default()
                .event("thinking")
                .json_data(ThinkingData { step: step.clone() })
                .unwrap());
        }

        // ── Final answer ──────────────────────────────────────────────────────
        match call_gemini_answer(
            &gemini_http_client,
            &gemini_api_key,
            &gemini_model,
            &enhanced_prompt,
            &user_message,
        ).await {
            Ok(content) => {
                if !content.trim().is_empty() {
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("response")
                        .json_data(ResponseData { content })
                        .unwrap());
                }
            }
            Err(e) => {
                tracing::error!("Gemini answer error: {}", e);
                yield Ok::<Event, Infallible>(Event::default()
                    .event("response")
                    .json_data(ResponseData {
                        content: "I couldn't generate a response right now. Please try again.".to_string()
                    })
                    .unwrap());
            }
        }

        // ── Sources ───────────────────────────────────────────────────────────
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

        yield Ok::<Event, Infallible>(Event::default()
            .event("done")
            .json_data(serde_json::json!({"status": "complete"}))
            .unwrap());
    };

    Sse::new(stream)
}

// ── Pass 1: Symptom normalization ─────────────────────────────────────────────

async fn call_gemini_normalize(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    user_message: &str,
) -> anyhow::Result<NormalizedQuery> {
    let prompt = format!(
        "You are a clinical terminology assistant. \
         Extract and normalize the patient's described symptoms into precise clinical terms \
         that are optimal for embedding-based similarity search against a rare disease database.\n\n\
         Return ONLY strict JSON with this exact schema:\n\
         {{\"clinical_query\": \"concise clinical description for embedding\", \"key_symptoms\": [\"symptom1\", \"symptom2\"]}}\n\n\
         Rules:\n\
         - clinical_query: 1-3 sentences using medical terminology (e.g. 'proximal muscle weakness' not 'arms are weak')\n\
         - key_symptoms: 3-7 individual normalized symptoms as strings\n\
         - No markdown, no explanation, output JSON only.\n\n\
         Patient description:\n{}",
        user_message
    );

    let req_body = GeminiGenerateRequest {
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart { text: prompt }],
        }],
        generation_config: GeminiGenerationConfig { temperature: 0.1 },
    };

    let res = http_client
        .post(gemini_endpoint(model, api_key))
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("Gemini normalize error {}: {}", status, body));
    }

    let content = extract_gemini_text(&body)?;
    let json_payload = extract_json_object(&content).unwrap_or(content);
    let output: NormalizedQuery = serde_json::from_str(&json_payload)
        .map_err(|e| anyhow::anyhow!("Failed to parse normalize JSON: {} | content: {}", e, json_payload))?;

    tracing::info!("Normalized clinical query: {}", output.clinical_query);
    Ok(output)
}

// ── Pass 2: Candidate selection ───────────────────────────────────────────────

async fn call_gemini_select(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    user_message: &str,
    candidates: &[(String, f32, crate::rag::vector_store::DocumentMetadata)],
) -> anyhow::Result<CandidateSelection> {
    let candidate_list = candidates
        .iter()
        .enumerate()
        .map(|(i, (text, score, meta))| {
            let label = parse_condition_from_text(text)
                .map(|(name, _)| name)
                .or_else(|| meta.orpha_code.as_ref().map(|c| format!("Orpha {}", c)))
                .unwrap_or_else(|| "Unknown".to_string());
            let orpha = meta.orpha_code.as_deref().unwrap_or("?");
            let snippet: String = text.chars().take(300).collect();
            format!("[{}] {} (Orpha: {}) — similarity {:.2}\n{}", i, label, orpha, score, snippet)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "You are a rare disease diagnostic assistant. \
         A patient described their symptoms and we retrieved {} candidate conditions from a vector database.\n\n\
         Patient description:\n{}\n\n\
         Candidates (0-indexed):\n{}\n\n\
         Pick the single best matching condition for this patient.\n\
         Return ONLY strict JSON:\n\
         {{\"selected_index\": <integer 0 to {}>, \"reasoning\": \"one sentence why\"}}\n\
         No markdown, no explanation. JSON only.",
        candidates.len(),
        user_message,
        candidate_list,
        candidates.len().saturating_sub(1)
    );

    let req_body = GeminiGenerateRequest {
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart { text: prompt }],
        }],
        generation_config: GeminiGenerationConfig { temperature: 0.1 },
    };

    let res = http_client
        .post(gemini_endpoint(model, api_key))
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("Gemini select error {}: {}", status, body));
    }

    let content = extract_gemini_text(&body)?;
    let json_payload = extract_json_object(&content).unwrap_or(content);
    let mut output: CandidateSelection = serde_json::from_str(&json_payload)
        .map_err(|e| anyhow::anyhow!("Failed to parse select JSON: {} | content: {}", e, json_payload))?;

    // Guard: clamp index to valid range
    if output.selected_index >= candidates.len() {
        tracing::warn!(
            "AI returned out-of-range selected_index {}, clamping to 0",
            output.selected_index
        );
        output.selected_index = 0;
    }

    tracing::info!(
        "AI selected candidate {} with reasoning: {}",
        output.selected_index,
        output.reasoning
    );
    Ok(output)
}

// ── Decorative thinking steps ─────────────────────────────────────────────────

async fn call_gemini_thinking(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    context_prompt: &str,
    user_message: &str,
) -> anyhow::Result<ThinkingOnlyOutput> {
    let prompt = format!(
        "You are a medical assistant for a hackathon demo. Return ONLY strict JSON with this schema:\n\
         {{\"thinking_steps\": [\"short step\", \"short step\"]}}\n\
         Use 3-6 concise UI-friendly steps, no hidden chain-of-thought.\n\n\
         Context:\n{}\n\nUser message:\n{}",
        context_prompt, user_message
    );

    let req_body = GeminiGenerateRequest {
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart { text: prompt }],
        }],
        generation_config: GeminiGenerationConfig { temperature: 0.2 },
    };

    let res = http_client
        .post(gemini_endpoint(model, api_key))
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("Gemini error {}: {}", status, body));
    }

    let content = extract_gemini_text(&body)?;
    let json_payload = extract_json_object(&content).unwrap_or(content);
    let output: ThinkingOnlyOutput = serde_json::from_str(&json_payload)
        .map_err(|e| anyhow::anyhow!("Failed to parse thinking JSON: {} | content: {}", e, json_payload))?;

    Ok(output)
}

// ── Final answer ──────────────────────────────────────────────────────────────

async fn call_gemini_answer(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    context_prompt: &str,
    user_message: &str,
) -> anyhow::Result<String> {
    let prompt = format!(
        "You are a medical assistant. The AI pipeline has already selected the best matching \
         rare disease from a vector database. Use the SELECTED BEST MATCH to formulate your answer.\n\n\
         If no match was found, say you cannot identify a likely condition and provide general next steps.\n\n\
         Output format (use these exact headers):\n\
         Most likely condition: <single condition name> (Orpha code if available)\n\
         Reasons:\n- <reason>\n- <reason>\n\
         Next steps:\n- <action>\n- <action>\n\
         Disclaimer: This is not a medical diagnosis. Please consult a qualified physician.\n\n\
         Do not list multiple conditions. Be concise.\n\n\
         {}\n\nUser message:\n{}",
        context_prompt, user_message
    );

    let req_body = GeminiGenerateRequest {
        contents: vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart { text: prompt }],
        }],
        generation_config: GeminiGenerationConfig { temperature: 0.2 },
    };

    let res = http_client
        .post(gemini_endpoint(model, api_key))
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;
    if !status.is_success() {
        return Err(anyhow::anyhow!("Gemini error {}: {}", status, body));
    }

    extract_gemini_text(&body)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn gemini_endpoint(model: &str, api_key: &str) -> String {
    format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    )
}

fn extract_gemini_text(body: &str) -> anyhow::Result<String> {
    let parsed: GeminiGenerateResponse = serde_json::from_str(body)
        .map_err(|e| anyhow::anyhow!("Failed to parse Gemini response: {} | body: {}", e, body))?;

    let text = parsed
        .candidates
        .first()
        .and_then(|c| {
            let combined = c.content.parts.iter()
                .filter_map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("");
            if combined.is_empty() { None } else { Some(combined) }
        })
        .ok_or_else(|| anyhow::anyhow!("Gemini returned no text candidates"))?;

    Ok(text)
}

fn extract_json_object(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(text[start..=end].to_string())
}

fn build_rag_context(results: &[(String, f32, crate::rag::vector_store::DocumentMetadata)]) -> String {
    if results.is_empty() {
        return String::new();
    }

    results
        .iter()
        .enumerate()
        .map(|(i, (text, score, metadata))| {
            let source = match metadata.source_type.as_str() {
                "orphanet" => format!("Orphanet: {}", metadata.source_id),
                "pdf"      => format!("PDF: {}", metadata.source_id),
                "image"    => format!("Image: {}", metadata.source_id),
                _          => metadata.source_id.clone(),
            };
            format!(
                "[{}] Source: {} (Relevance: {:.2})\n{}",
                i + 1, source, score,
                text.chars().take(400).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn parse_condition_from_text(text: &str) -> Option<(String, Option<String>)> {
    let first_line = text.lines().next()?.trim();
    if !first_line.starts_with("Disease:") {
        return None;
    }

    let rest = first_line.trim_start_matches("Disease:").trim();
    if rest.is_empty() {
        return None;
    }

    if let Some(orpha_idx) = rest.find("(Orpha:") {
        let name = rest[..orpha_idx].trim().to_string();
        let tail = &rest[orpha_idx + "(Orpha:".len()..];
        let code = tail.split(')').next().map(|s| s.trim().to_string());
        if name.is_empty() {
            return None;
        }
        return Some((name, code));
    }

    Some((rest.to_string(), None))
}
