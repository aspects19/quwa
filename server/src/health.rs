use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthCheckResponse {
    status: String,
}

pub async fn health_check() -> Json<HealthCheckResponse> {
    let response = HealthCheckResponse {
        status: "ok".to_string(),
    };
    Json(response)
}