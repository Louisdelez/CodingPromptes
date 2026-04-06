use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub email: String,
    pub exp: usize,
}

pub fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "prompt-ide-dev-secret-change-in-production".into())
}

pub fn create_token(user_id: &str, email: &str) -> Result<String, String> {
    let exp = chrono::Utc::now().timestamp() as usize + 7 * 24 * 3600; // 7 days
    let claims = Claims { sub: user_id.to_string(), email: email.to_string(), exp };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret().as_bytes()))
        .map_err(|e| e.to_string())
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    decode::<Claims>(token, &DecodingKey::from_secret(jwt_secret().as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| e.to_string())
}

/// Axum extractor: extracts authenticated user_id from Bearer token
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
}

impl<S> FromRequestParts<S> for AuthUser
where S: Send + Sync {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".into()))?;

        let token = header.strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization format".into()))?;

        let claims = verify_token(token)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {e}")))?;

        Ok(AuthUser { user_id: claims.sub, email: claims.email })
    }
}
