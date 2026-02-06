use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;

// Appwrite JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppwriteClaims {
    #[serde(rename = "$id")]
    pub user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

impl Display for AppwriteClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User ID: {}", self.user_id)
    }
}

// Extractor for Appwrite JWT from request
impl<S> FromRequestParts<S> for AppwriteClaims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingToken)?;

        // Validate JWT with Appwrite API
        let claims = validate_with_appwrite_api(bearer.token())
            .await
            .map_err(|e| {
                tracing::error!("Appwrite API validation failed: {}", e);
                AuthError::InvalidToken
            })?;

        Ok(claims)
    }
}

// Validate JWT by calling Appwrite API
async fn validate_with_appwrite_api(token: &str) -> anyhow::Result<AppwriteClaims> {
    let appwrite_endpoint = std::env::var("APPWRITE_ENDPOINT")
        .unwrap_or_else(|_| "https://cloud.appwrite.io/v1".to_string());
    let project_id = std::env::var("APPWRITE_PROJECT_ID")
        .map_err(|_| anyhow::anyhow!("APPWRITE_PROJECT_ID not set"))?;
    
    // Call Appwrite API to validate the JWT
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/account", appwrite_endpoint))
        .header("X-Appwrite-Project", &project_id)
        .header("X-Appwrite-JWT", token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call Appwrite API: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Appwrite API returned error {}: {}",
            status,
            error_text
        ));
    }
    
    // Parse user info from response
    let user: serde_json::Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse Appwrite response: {}", e))?;
    
    Ok(AppwriteClaims {
        user_id: user["$id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user ID in Appwrite response"))?
            .to_string(),
        email: user["email"].as_str().map(|s| s.to_string()),
        name: user["name"].as_str().map(|s| s.to_string()),
    })
}

// Auth error types
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    ExpiredToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid authorization token"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token has expired"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

// Test endpoint to verify JWT validation works
pub async fn protected(claims: AppwriteClaims) -> Result<String, AuthError> {
    Ok(format!(
        "Welcome to the protected area!\n{}\nEmail: {:?}\nName: {:?}",
        claims,
        claims.email,
        claims.name
    ))
}