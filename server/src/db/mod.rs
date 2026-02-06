use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;

pub mod models;
pub mod queries;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    Ok(pool)
}
