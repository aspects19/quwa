pub mod health;
pub mod auth;
pub mod chat;
pub mod db;
pub mod rag;
pub mod processing;
pub mod media_ingestion;
pub mod request_counter;

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
    pub gemini_client: Arc<rig::providers::gemini::Client>,
    pub request_counter: request_counter::RequestCounter,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Quwa Medical Assistant Server...");

    // Get environment variables
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            tracing::warn!("DATABASE_URL not set, using default");
            "postgresql://localhost/quwa".to_string()
        });
    
    let gemini_api_key = std::env::var("GEMINI_API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("GEMINI_API_KEY not set");
            String::new()
        });

    // Initialize database
    tracing::info!("Connecting to database...");
    let db_pool = db::create_pool(&database_url).await?;
    tracing::info!("Database connected and migrations run successfully");

    // Initialize vector store
    tracing::info!("Initializing vector store...");
    let vector_store = Arc::new(rag::vector_store::RagVectorStore::new());

    // Initialize shared Gemini client
    tracing::info!("Initializing Gemini client...");
    let gemini_client = Arc::new(rig::providers::gemini::Client::new(&gemini_api_key)?);

    // Initialize processors
    let pdf_processor = Arc::new(processing::PdfProcessor::new(&gemini_api_key)?);
    let image_processor = Arc::new(processing::ImageProcessor::new(&gemini_api_key)?);

    // Create request counter for API tracking
    let request_counter = request_counter::RequestCounter::new();
    tracing::info!("Request counter initialized");

    // Create application state
    let state = AppState {
        db_pool,
        vector_store,
        pdf_processor,
        image_processor,
        gemini_client,
        request_counter,
    };

    // TODO: Bootstrap Orphadata
    // tracing::info!("Loading Orphadata into vector store...");
    // orphadata::ingestion::ingest_to_vector_store(&state.vector_store, &gemini_client, &state.db_pool).await?;

    // Build router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/protected", get(auth::protected))
        .route("/api/chat", post(chat_handler))
        .route("/api/upload", post(media_ingestion::handle_file_upload))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;

    Ok(())
}