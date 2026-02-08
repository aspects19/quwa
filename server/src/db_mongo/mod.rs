pub mod models;
pub mod queries;

use anyhow::{Result, Context};
use mongodb::{Client, Database};

/// Create MongoDB connection
pub async fn create_client(uri: &str) -> Result<Client> {
    let client = Client::with_uri_str(uri)
        .await
        .context("Failed to connect to MongoDB")?;
    
    // Ping to verify connection
    client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1})
        .await
        .context("Failed to ping MongoDB")?;
    
    tracing::info!("Successfully connected to MongoDB");
    Ok(client)
}

/// Get database handle
pub fn get_database(client: &Client, db_name: &str) -> Database {
    client.database(db_name)
}
