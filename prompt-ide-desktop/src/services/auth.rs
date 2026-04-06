use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub salt: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{salt}:{password}"));
    hex::encode(hasher.finalize())
}

pub fn generate_salt() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}
