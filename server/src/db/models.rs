use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub appwrite_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UploadedFile {
    pub id: Uuid,
    pub user_id: i32,
    pub file_name: String,
    pub file_type: String,
    pub mime_type: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub appwrite_file_id: String,
    pub appwrite_bucket_id: String,
    pub processing_status: String,
    pub upload_date: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmbeddingMetadata {
    pub id: Uuid,
    pub file_id: Uuid,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub embedding_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrphadataDisease {
    pub id: i32,
    pub orpha_code: String,
    pub disease_name: String,
    pub description: Option<String>,
    pub symptoms: Option<String>,
    pub diagnostic_criteria: Option<String>,
    pub prevalence: Option<String>,
    pub category: Option<String>,
    pub last_updated: DateTime<Utc>,
}
