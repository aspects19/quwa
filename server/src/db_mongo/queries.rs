use anyhow::Result;
use mongodb::{Database, bson::{doc, oid::ObjectId}};
use super::models::*;

pub async fn get_or_create_user(
    db: &Database,
    appwrite_id: &str,
    email: Option<&str>,
    name: Option<&str>,
) -> Result<User> {
    let collection = db.collection::<User>("users");
    
    // Try to find existing user
    if let Some(user) = collection
        .find_one(doc! { "appwrite_id": appwrite_id })
        .await?
    {
        // Update timestamp
        collection
            .update_one(
                doc! { "appwrite_id": appwrite_id },
                doc! { "$set": { "updated_at": mongodb::bson::DateTime::now() } },
            )
            .await?;
        return Ok(user);
    }
    
    // Create new user
    let mut user = User::new(
        appwrite_id.to_string(),
        email.map(|s| s.to_string()),
        name.map(|s| s.to_string()),
    );
    
    let result = collection.insert_one(&user).await?;
    user.id = Some(result.inserted_id.as_object_id().unwrap());
    
    Ok(user)
}

pub async fn create_uploaded_file(
    db: &Database,
    file: UploadedFile,
) -> Result<UploadedFile> {
    let collection = db.collection::<UploadedFile>("uploaded_files");
    
    let result = collection.insert_one(&file).await?;
    let mut file = file;
    file.id = Some(result.inserted_id.as_object_id().unwrap());
    
    Ok(file)
}

pub async fn update_file_status(
    db: &Database,
    file_id: ObjectId,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    let collection = db.collection::<UploadedFile>("uploaded_files");
    
    collection
        .update_one(
            doc! { "_id": file_id },
            doc! {
                "$set": {
                    "processing_status": status,
                    "error_message": error,
                    "processed_at": mongodb::bson::DateTime::now(),
                }
            },
        )
        .await?;
    
    Ok(())
}

pub async fn get_user_files(
    db: &Database,
    user_id: ObjectId,
) -> Result<Vec<UploadedFile>> {
    let collection = db.collection::<UploadedFile>("uploaded_files");
    
    let mut cursor = collection
        .find(doc! { "user_id": user_id })
        .sort(doc! { "upload_date": -1 })
        .await?;
    
    let mut files = Vec::new();
    while cursor.advance().await? {
        files.push(cursor.deserialize_current()?);
    }
    
    Ok(files)
}

pub async fn get_file_by_id(
    db: &Database,
    file_id: ObjectId,
) -> Result<Option<UploadedFile>> {
    let collection = db.collection::<UploadedFile>("uploaded_files");
    
    let file = collection
        .find_one(doc! { "_id": file_id })
        .await?;
    
    Ok(file)
}

pub async fn create_embedding_metadata(
    db: &Database,
    file_id: ObjectId,
    chunk_index: i32,
    chunk_text: &str,
    embedding_id: &str,
) -> Result<EmbeddingMetadata> {
    let collection = db.collection::<EmbeddingMetadata>("embeddings_metadata");
    
    let metadata = EmbeddingMetadata {
        id: None,
        file_id,
        chunk_index,
        chunk_text: chunk_text.to_string(),
        embedding_id: embedding_id.to_string(),
        created_at: mongodb::bson::DateTime::now(),
    };
    
    let result = collection.insert_one(&metadata).await?;
    let mut metadata = metadata;
    metadata.id = Some(result.inserted_id.as_object_id().unwrap());
    
    Ok(metadata)
}
