use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub appwrite_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: mongodb::bson::DateTime,
    pub updated_at: mongodb::bson::DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub file_name: String,
    pub file_type: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub appwrite_file_id: String,
    pub appwrite_bucket_id: String,
    pub processing_status: String,
    pub error_message: Option<String>,
    pub upload_date: mongodb::bson::DateTime,
    pub processed_at: Option<mongodb::bson::DateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub file_id: ObjectId,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub embedding_id: String,
    pub created_at: mongodb::bson::DateTime,
}

impl User {
    pub fn new(appwrite_id: String, email: Option<String>, name: Option<String>) -> Self {
        let now = mongodb::bson::DateTime::now();
        Self {
            id: None,
            appwrite_id,
            email,
            name,
            created_at: now,
            updated_at: now,
        }
    }
}

impl UploadedFile {
    pub fn new(
        user_id: ObjectId,
        file_name: String,
        file_type: String,
        mime_type: String,
        file_size_bytes: i64,
        appwrite_file_id: String,
        appwrite_bucket_id: String,
    ) -> Self {
        Self {
            id: None,
            user_id,
            file_name,
            file_type,
            mime_type,
            file_size_bytes,
            appwrite_file_id,
            appwrite_bucket_id,
            processing_status: "pending".to_string(),
            error_message: None,
            upload_date: mongodb::bson::DateTime::now(),
            processed_at: None,
        }
    }
}
