pub mod health;
pub mod auth;
pub mod chat;
pub mod db;
pub mod rag;
pub mod processing;
pub mod media_ingestion;
pub mod request_counter;
pub mod embeddings;
pub mod orphanet_loader;

use anyhow::Result;
use axum::{Router, routing::{get, post}};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use std::sync::Arc;

use crate::{chat::chat_handler, health::health_check};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub vector_store: Arc<rag::vector_store::RagVectorStore>,
    pub pdf_processor: Arc<processing::PdfProcessor>,
    pub image_processor: Arc<processing::ImageProcessor>,
    pub gemini_http_client: reqwest::Client,
    pub gemini_api_key: Arc<String>,
    pub gemini_model: Arc<String>,
    pub embedding_service: Arc<embeddings::LocalEmbeddingService>,
    pub request_counter: request_counter::RequestCounter,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Quwa Medical Assistant Server...");

    // Get environment variables
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set, Set it in .env file");

    let gemini_api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY not set, Set it in .env file");
    let gemini_model = std::env::var("GEMINI_MODEL")
        .unwrap_or_else(|_| "gemini-2.0-flash".to_string());

    // Initialize PostgreSQL
    tracing::info!("Connecting to PostgreSQL...");
    let db_pool = db::create_pool(&database_url).await?;
    tracing::info!("PostgreSQL connected successfully");

    // Initialize local embedding service
    tracing::info!("Initializing local embedding service...");
    let embedding_service = Arc::new(embeddings::LocalEmbeddingService::new()?);
    tracing::info!("Embedding service ready (dimension: {})", embedding_service.dimension());

    // Initialize PostgreSQL vector store with pgvector
    tracing::info!("Initializing PostgreSQL vector store...");
    let vector_store = Arc::new(
        rag::vector_store::RagVectorStore::new(&db_pool, embedding_service.clone()).await?
    );
    let vector_count = vector_store.count().await;
    tracing::info!("Vector store initialized ({} existing documents)", vector_count);

    // Initialize Gemini client settings
    tracing::info!("Initializing Gemini HTTP client with model {}...", gemini_model);
    let gemini_http_client = reqwest::Client::new();

    // Initialize processors with local embeddings
    let pdf_processor = Arc::new(processing::PdfProcessor::new(embedding_service.clone())?);
    let image_processor = Arc::new(processing::ImageProcessor::new(embedding_service.clone())?);

    // Create request counter for API tracking
    let request_counter = request_counter::RequestCounter::new();
    tracing::info!("Request counter initialized");

    // Create application state
    let state = AppState {
        db_pool,
        vector_store: vector_store.clone(),
        pdf_processor,
        image_processor,
        gemini_http_client,
        gemini_api_key: Arc::new(gemini_api_key),
        gemini_model: Arc::new(gemini_model),
        embedding_service: embedding_service.clone(),
        request_counter,
    };

    // Load Orphanet data if enabled
    let load_orphanet = std::env::var("LOAD_ORPHANET")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    if load_orphanet {
        tracing::info!("Loading Orphanet dataset...");
        match orphanet_loader::load_orphanet_from_env(&vector_store, &embedding_service).await {
            Ok(count) => {
                tracing::info!("✓ Loaded {} Orphanet disorders", count);
            }
            Err(e) => {
                tracing::error!("Failed to load Orphanet data: {}", e);
                tracing::warn!("Continuing without Orphanet data...");
            }
        }
    } else {
        tracing::info!("Orphanet loading disabled (set LOAD_ORPHANET=true to enable)");
    }

    // Build router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/protected", get(auth::protected))
        .route("/api/chat", post(chat_handler))
        .route("/api/upload", post(media_ingestion::handle_file_upload))
        .route("/api/vector/inspect", post(rag::inspect::inspect_vectors))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);
    println!("Server running at http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
