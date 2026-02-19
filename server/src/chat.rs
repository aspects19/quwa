use axum::{
    extract::{Json, State},
    response::sse::{Event, Sse},
};
use futures_util::{stream::Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

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

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<OpenAiResponseFormat>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiResponseFormat {
    #[serde(rename = "type")]
    type_name: String,
    json_schema: OpenAiJsonSchema,
}

#[derive(Debug, Serialize)]
struct OpenAiJsonSchema {
    name: String,
    strict: bool,
    schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ThinkingOnlyOutput {
    thinking_steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct TopMatch {
    label: Option<String>,
    similarity: Option<f32>,
    orpha_code: Option<String>,
}

impl TopMatch {
    fn to_prompt_line(&self) -> String {
        match (&self.label, self.similarity, &self.orpha_code) {
            (Some(label), Some(sim), Some(code)) => {
                format!("TOP VECTOR MATCH: {} (Orpha: {}) with similarity {:.2}.", label, code, sim)
            }
            (Some(label), Some(sim), None) => {
                format!("TOP VECTOR MATCH: {} with similarity {:.2}.", label, sim)
            }
            (Some(label), None, _) => {
                format!("TOP VECTOR MATCH: {}.", label)
            }
            _ => "TOP VECTOR MATCH: none.".to_string(),
        }
    }
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
    let openai_http_client = state.openai_http_client.clone();
    let openai_api_key = state.openai_api_key.clone();
    let openai_model = state.openai_model.clone();
    let embedding_service = state.embedding_service.clone();
    let user_id = claims.user_id.clone();
    let request_counter = state.request_counter.clone();

    // Create events stream with RAG integration
    let stream = async_stream::stream! {
        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Analyzing patient query and symptoms...".to_string()
            })
            .unwrap());

        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Generating semantic representation...".to_string()
            })
            .unwrap());

        let enable_embeddings = std::env::var("ENABLE_EMBEDDINGS")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        let query_embedding = if enable_embeddings {
            match embedding_service.embed_text(&user_message).await {
                Ok(emb) => emb,
                Err(e) => {
                    tracing::error!("Local embedding generation failed: {}", e);
                    yield Ok::<Event, Infallible>(Event::default()
                        .event("thinking")
                        .json_data(ThinkingData {
                            step: "Embedding failed, proceeding without context search...".to_string()
                        })
                        .unwrap());
                    vec![0.0; 384]
                }
            }
        } else {
            tracing::info!("Embeddings disabled via ENABLE_EMBEDDINGS=false");
            vec![0.0; 384]
        };

        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Searching medical knowledge base and patient records...".to_string()
            })
            .unwrap());

        let user_files = match crate::db::queries::get_user_files(&db_pool,
            crate::db::queries::get_or_create_user(&db_pool, &user_id, None, None)
                .await
                .map(|u| u.id)
                .unwrap_or(0)
        ).await {
            Ok(files) => files,
            Err(_) => vec![]
        };

        let rag_results = match vector_store.search(query_embedding, 5).await {
            Ok(results) => results,
            Err(e) => {
                tracing::error!("Vector search failed: {}", e);
                vec![]
            }
        };

        let _strategy = context_strategy::determine_strategy(&user_files, &user_message);

        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: format!("Found {} relevant medical references", rag_results.len())
            })
            .unwrap());

        let top_match = extract_top_condition(&rag_results);
        if let Some(top_match_label) = top_match.label.clone() {
            yield Ok::<Event, Infallible>(Event::default()
                .event("thinking")
                .json_data(ThinkingData {
                    step: format!("Top match: {}", top_match_label)
                })
                .unwrap());
        }

        let context = build_rag_context(&rag_results);

        yield Ok::<Event, Infallible>(Event::default()
            .event("thinking")
            .json_data(ThinkingData {
                step: "Calling OpenAI model...".to_string()
            })
            .unwrap());

        let top_match_note = top_match.to_prompt_line();

        let enhanced_prompt = if !context.is_empty() {
            format!(
                "RELEVANT MEDICAL KNOWLEDGE:\n{}\n\n{}\n\nPATIENT QUERY:\n{}",
                context, top_match_note, user_message
            )
        } else {
            format!(
                "PATIENT QUERY:\n{}\n\nNo specific retrieved records were found for this query.\n\n{}",
                user_message, top_match_note
            )
        };

        request_counter.log_chat_request(
            &format!("OpenAI chat | User query: {}", user_message.chars().take(50).collect::<String>())
        );

        const MAX_THINKING_RETRIES: u32 = 3;
        const THINKING_BASE_DELAY_MS: u64 = 1200;

        let mut thinking_steps: Vec<String> = Vec::new();
        for attempt in 0..MAX_THINKING_RETRIES {
            match call_openai_thinking(
                &openai_http_client,
                &openai_api_key,
                &openai_model,
                &enhanced_prompt,
                &user_message,
            ).await {
                Ok(output) => {
                    thinking_steps = output.thinking_steps;
                    break;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let rate_limited = error_msg.contains("429") || error_msg.contains("rate_limit") || error_msg.contains("Rate limit");

                    if rate_limited && attempt < MAX_THINKING_RETRIES - 1 {
                        let jitter = ((attempt as u64 + 1) * 173) % 500;
                        let delay = THINKING_BASE_DELAY_MS * 2_u64.pow(attempt) + jitter;

                        tracing::warn!(
                            "OpenAI thinking rate limit hit, retrying in {}ms (attempt {}/{})",
                            delay,
                            attempt + 1,
                            MAX_THINKING_RETRIES
                        );

                        yield Ok::<Event, Infallible>(Event::default()
                            .event("thinking")
                            .json_data(ThinkingData {
                                step: format!("Rate limit hit, retrying in {}s...", delay / 1000)
                            })
                            .unwrap());

                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        continue;
                    }

                    tracing::warn!("Could not generate model thinking steps: {}", e);
                    break;
                }
            }
        }

        for step in thinking_steps.iter().take(8) {
            yield Ok::<Event, Infallible>(Event::default()
                .event("thinking")
                .json_data(ThinkingData { step: step.clone() })
                .unwrap());
        }

        match start_openai_answer_stream(
            &openai_http_client,
            &openai_api_key,
            &openai_model,
            &enhanced_prompt,
            &user_message,
        ).await {
            Ok(res) => {
                let mut upstream = res.bytes_stream();
                let mut buffer = String::new();
                let mut stream_done = false;

                while let Some(item) = upstream.next().await {
                    let bytes = match item {
                        Ok(b) => b,
                        Err(e) => {
                            tracing::error!("OpenAI stream chunk read error: {}", e);
                            break;
                        }
                    };

                    let text = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&text);

                    while let Some(idx) = buffer.find('\n') {
                        let line = buffer[..idx].trim().to_string();
                        buffer = buffer[idx + 1..].to_string();

                        if !line.starts_with("data: ") {
                            continue;
                        }

                        let payload = line.trim_start_matches("data: ").trim();
                        if payload == "[DONE]" {
                            stream_done = true;
                            break;
                        }

                        if payload.is_empty() {
                            continue;
                        }

                        let parsed: OpenAiStreamChunk = match serde_json::from_str(payload) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };

                        if let Some(content) = parsed
                            .choices
                            .first()
                            .and_then(|c| c.delta.content.clone())
                        {
                            if content.is_empty() {
                                continue;
                            }
                            yield Ok::<Event, Infallible>(Event::default()
                                .event("response")
                                .json_data(ResponseData {
                                    content,
                                })
                                .unwrap());
                        }
                    }

                    if stream_done {
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::error!("OpenAI streaming error: {}", e);
                yield Ok::<Event, Infallible>(Event::default()
                    .event("response")
                    .json_data(ResponseData {
                        content: "I couldn't generate a response right now. Please try again in a few seconds.".to_string()
                    })
                    .unwrap());
            }
        }

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

        yield Ok::<Event, Infallible>(Event::default()
            .event("done")
            .json_data(serde_json::json!({"status": "complete"}))
            .unwrap());
    };

    Sse::new(stream)
}

async fn call_openai_thinking(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    context_prompt: &str,
    user_message: &str,
) -> anyhow::Result<ThinkingOnlyOutput> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "thinking_steps": {
                "type": "array",
                "items": { "type": "string" },
                "description": "A short list of concise reasoning steps for the user interface."
            }
        },
        "required": ["thinking_steps"],
        "additionalProperties": false
    });

    let system_prompt = "You are a medical assistant for a hackathon demo. Return concise UI-friendly reasoning as thinking_steps (3-6 short lines). Do not include hidden chain-of-thought details.";

    let req_body = OpenAiChatRequest {
        model: model.to_string(),
        messages: vec![
            OpenAiMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenAiMessage {
                role: "user".to_string(),
                content: format!("{}\n\nUser message:\n{}", context_prompt, user_message),
            }
        ],
        temperature: 0.2,
        stream: None,
        response_format: Some(OpenAiResponseFormat {
            type_name: "json_schema".to_string(),
            json_schema: OpenAiJsonSchema {
                name: "medical_chat_output".to_string(),
                strict: true,
                schema,
            }
        }),
    };

    let res = http_client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;

    if !status.is_success() {
        return Err(anyhow::anyhow!("OpenAI error {}: {}", status, body));
    }

    let parsed: OpenAiChatResponse = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Failed to parse OpenAI response: {} | body: {}", e, body))?;

    let content = parsed
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow::anyhow!("OpenAI returned no choices"))?;

    let output: ThinkingOnlyOutput = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse thinking JSON output: {} | content: {}", e, content))?;

    Ok(output)
}

async fn start_openai_answer_stream(
    http_client: &reqwest::Client,
    api_key: &str,
    model: &str,
    context_prompt: &str,
    user_message: &str,
) -> anyhow::Result<reqwest::Response> {
    let req_body = OpenAiChatRequest {
        model: model.to_string(),
        messages: vec![
            OpenAiMessage {
                role: "system".to_string(),
                content: "You are a medical assistant for a hackathon demo. Use the top vector match if provided. If the top vector match is none, say you cannot identify a likely condition and provide general next steps. Output format:\nMost likely condition: <single condition name> (Orpha code if provided)\nReasons:\n- <short reason>\n- <short reason>\nNext steps:\n- <action>\n- <action>\nInclude a brief disclaimer that this is not a diagnosis. Do not list multiple conditions.".to_string(),
            },
            OpenAiMessage {
                role: "user".to_string(),
                content: format!("{}\n\nUser message:\n{}", context_prompt, user_message),
            },
        ],
        temperature: 0.2,
        stream: Some(true),
        response_format: None,
    };

    let res = http_client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("OpenAI stream error {}: {}", status, body));
    }

    Ok(res)
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
                "orphanet" => format!("Orphanet: {}", metadata.source_id),
                "pdf" => format!("PDF: {}", metadata.source_id),
                "image" => format!("Image: {}", metadata.source_id),
                _ => metadata.source_id.clone(),
            };

            format!(
                "[{}] Source: {} (Relevance: {:.2})\n{}",
                i + 1,
                source,
                score,
                text.chars().take(600).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn extract_top_condition(
    results: &[(String, f32, crate::rag::vector_store::DocumentMetadata)],
) -> TopMatch {
    let mut top_match = TopMatch::default();
    let Some((text, similarity, metadata)) = results.first() else {
        return top_match;
    };

    if let Some((name, orpha_code)) = parse_condition_from_text(text) {
        top_match.label = Some(name);
        top_match.orpha_code = orpha_code.or(metadata.orpha_code.clone());
    } else if let Some(orpha_code) = metadata.orpha_code.clone() {
        top_match.label = Some(format!("Orpha {}", orpha_code));
        top_match.orpha_code = Some(orpha_code);
    }

    top_match.similarity = Some(*similarity);
    top_match
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
