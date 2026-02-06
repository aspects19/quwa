use sqlx::PgPool;
use anyhow::Result;
use uuid::Uuid;
use super::models::*;

pub async fn get_or_create_user(
    pool: &PgPool,
    appwrite_id: &str,
    email: Option<&str>,
    name: Option<&str>,
) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (appwrite_id, email, name) 
         VALUES ($1, $2, $3)
         ON CONFLICT (appwrite_id) DO UPDATE SET updated_at = NOW()
         RETURNING *"
    )
    .bind(appwrite_id)
    .bind(email)
    .bind(name)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}

pub async fn create_uploaded_file(pool: &PgPool, file: &UploadedFile) -> Result<UploadedFile> {
    let file = sqlx::query_as::<_, UploadedFile>(
        "INSERT INTO uploaded_files (id, user_id, file_name, file_type, mime_type, file_size_bytes, appwrite_file_id, appwrite_bucket_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *"
    )
    .bind(file.id)
    .bind(file.user_id)
    .bind(&file.file_name)
    .bind(&file.file_type)
    .bind(&file.mime_type)
    .bind(file.file_size_bytes)
    .bind(&file.appwrite_file_id)
    .bind(&file.appwrite_bucket_id)
    .fetch_one(pool)
    .await?;
    
    Ok(file)
}

pub async fn update_file_status(
    pool: &PgPool,
    file_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "UPDATE uploaded_files SET processing_status = $1, error_message = $2, processed_at = NOW() WHERE id = $3"
    )
    .bind(status)
    .bind(error)
    .bind(file_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_user_files(pool: &PgPool, user_id: i32) -> Result<Vec<UploadedFile>> {
    let files = sqlx::query_as::<_, UploadedFile>(
        "SELECT * FROM uploaded_files WHERE user_id = $1 ORDER BY upload_date DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    
    Ok(files)
}

pub async fn get_file_by_id(pool: &PgPool, file_id: Uuid) -> Result<Option<UploadedFile>> {
    let file = sqlx::query_as::<_, UploadedFile>(
        "SELECT * FROM uploaded_files WHERE id = $1"
    )
    .bind(file_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(file)
}

pub async fn create_embedding_metadata(
    pool: &PgPool,
    file_id: Uuid,
    chunk_index: i32,
    chunk_text: &str,
    embedding_id: &str,
) -> Result<EmbeddingMetadata> {
    let metadata = sqlx::query_as::<_, EmbeddingMetadata>(
        "INSERT INTO embeddings_metadata (file_id, chunk_index, chunk_text, embedding_id)
         VALUES ($1, $2, $3, $4)
         RETURNING *"
    )
    .bind(file_id)
    .bind(chunk_index)
    .bind(chunk_text)
    .bind(embedding_id)
    .fetch_one(pool)
    .await?;
    
    Ok(metadata)
}
