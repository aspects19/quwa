pub mod health;
pub mod auth;
pub mod chat;

use anyhow::Result;
use axum::{Router, routing::{get, post}};
use tokio::net::TcpListener;

use crate::health::health_check;



#[tokio::main]
async fn main() -> Result<()> {

//     tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check));
        // .route("sign-up", post(sign_up))
        // .route("sign-in", post(sign_in))
        // .route("chat", post(chat));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app).await;

    Ok(())

}