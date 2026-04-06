use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::database::*;
use crate::jwt_auth::{create_token, AuthUser};
use crate::AppState;

// --- Request/Response types ---

#[derive(Deserialize)]
pub struct RegisterReq { pub email: String, pub password: String, pub display_name: String }

#[derive(Deserialize)]
pub struct LoginReq { pub email: String, pub password: String }

#[derive(Serialize)]
pub struct AuthResp { pub token: String, pub user: UserResp }

#[derive(Serialize)]
pub struct UserResp { pub id: String, pub email: String, pub display_name: String, pub avatar: String }

#[derive(Deserialize)]
pub struct CreateWorkspaceReq { pub name: String, pub description: Option<String>, pub color: String }

#[derive(Deserialize)]
pub struct CreateProjectReq { pub id: Option<String>, pub name: String, pub workspace_id: Option<String>, pub blocks_json: String, pub variables_json: Option<String>, pub framework: Option<String>, pub tags_json: Option<String> }

#[derive(Deserialize)]
pub struct UpdateProjectReq { pub name: Option<String>, pub workspace_id: Option<String>, pub blocks_json: Option<String>, pub variables_json: Option<String>, pub framework: Option<String>, pub tags_json: Option<String> }

#[derive(Deserialize)]
pub struct CreateVersionReq { pub blocks_json: String, pub variables_json: String, pub label: String }

#[derive(Deserialize)]
pub struct CreateExecutionReq { pub model: String, pub provider: String, pub prompt: String, pub response: String, pub tokens_in: i64, pub tokens_out: i64, pub cost: f64, pub latency_ms: i64 }

#[derive(Deserialize)]
pub struct CreateFrameworkReq { pub name: String, pub description: Option<String>, pub blocks_json: String }

#[derive(Deserialize)]
pub struct UpdateFrameworkReq { pub name: Option<String>, pub description: Option<String>, pub blocks_json: Option<String> }

#[derive(Deserialize)]
pub struct ConfigReq { pub config: std::collections::HashMap<String, String> }

#[derive(Deserialize)]
pub struct OAuthGoogleReq { pub token: String }

#[derive(Deserialize)]
pub struct OAuthGithubReq { pub code: String }

#[derive(Deserialize)]
pub struct PresenceReq { pub project_id: String }

#[derive(Serialize)]
pub struct PresenceUser { pub user_id: String, pub display_name: String }

fn now() -> i64 { chrono::Utc::now().timestamp_millis() }
fn new_id() -> String { uuid::Uuid::new_v4().to_string() }

fn hash_pw(password: &str) -> Result<String, String> {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    // Generate a random 16-byte salt encoded as base64
    let raw_salt: [u8; 16] = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        uuid::Uuid::new_v4().to_string().hash(&mut h);
        chrono::Utc::now().timestamp_nanos_opt().hash(&mut h);
        let a = h.finish().to_le_bytes();
        let mut h2 = DefaultHasher::new();
        uuid::Uuid::new_v4().to_string().hash(&mut h2);
        let b = h2.finish().to_le_bytes();
        let mut salt = [0u8; 16];
        salt[..8].copy_from_slice(&a);
        salt[8..].copy_from_slice(&b);
        salt
    };
    let salt = SaltString::encode_b64(&raw_salt).map_err(|e| e.to_string())?;
    Argon2::default().hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_pw(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordVerifier, PasswordHash};
    PasswordHash::new(hash).ok()
        .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

fn avatar_from_name(name: &str) -> String {
    let colors = ["#6366f1","#8b5cf6","#ec4899","#f43f5e","#f97316","#22c55e","#06b6d4","#3b82f6"];
    let idx = name.bytes().map(|b| b as usize).sum::<usize>() % colors.len();
    let initials: String = name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase();
    serde_json::json!({"color": colors[idx], "initials": if initials.is_empty() { "?".into() } else { initials }}).to_string()
}

// --- Routes ---

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Auth (no JWT required)
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        // Data (JWT required)
        .route("/workspaces", get(list_workspaces).post(create_workspace))
        .route("/workspaces/{id}", delete(delete_workspace))
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/{id}", put(update_project).delete(delete_project))
        .route("/projects/{id}/versions", get(list_versions).post(create_version))
        .route("/projects/{id}/executions", get(list_executions).post(create_execution))
        .route("/frameworks", get(list_frameworks).post(create_framework))
        .route("/frameworks/{id}", put(update_framework).delete(delete_framework))
        .route("/config", get(get_config).put(set_config))
        // OAuth (no JWT required)
        .route("/auth/oauth/google", post(oauth_google))
        .route("/auth/oauth/github", post(oauth_github))
        // Presence (JWT required)
        .route("/presence", post(set_presence))
        .route("/presence/{project_id}", get(get_presence))
}

// --- Auth handlers ---

async fn register(State(state): State<Arc<AppState>>, Json(req): Json<RegisterReq>) -> Result<Json<AuthResp>, (StatusCode, String)> {
    if req.password.len() < 6 { return Err((StatusCode::BAD_REQUEST, "Password too short".into())); }
    if !req.email.contains('@') { return Err((StatusCode::BAD_REQUEST, "Invalid email".into())); }

    let db = state.db.lock().await;
    if db.get_user_by_email(&req.email.to_lowercase()).is_some() {
        return Err((StatusCode::CONFLICT, "EMAIL_EXISTS".into()));
    }

    let hash = hash_pw(&req.password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let user = DbUser {
        id: new_id(), email: req.email.to_lowercase(), display_name: req.display_name.clone(),
        password_hash: hash, avatar: avatar_from_name(&req.display_name), created_at: now(),
    };
    db.create_user(&user).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let token = create_token(&user.id, &user.email).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(AuthResp { token, user: UserResp { id: user.id, email: user.email, display_name: user.display_name, avatar: user.avatar } }))
}

async fn login(State(state): State<Arc<AppState>>, Json(req): Json<LoginReq>) -> Result<Json<AuthResp>, (StatusCode, String)> {
    let db = state.db.lock().await;
    let user = db.get_user_by_email(&req.email.to_lowercase())
        .ok_or((StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS".into()))?;

    if !verify_pw(&req.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS".into()));
    }

    let token = create_token(&user.id, &user.email).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(AuthResp { token, user: UserResp { id: user.id, email: user.email, display_name: user.display_name, avatar: user.avatar } }))
}

async fn me(State(state): State<Arc<AppState>>, auth: AuthUser) -> Result<Json<UserResp>, (StatusCode, String)> {
    let db = state.db.lock().await;
    let user = db.get_user_by_id(&auth.user_id).ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;
    Ok(Json(UserResp { id: user.id, email: user.email, display_name: user.display_name, avatar: user.avatar }))
}

// --- Workspace handlers ---

async fn list_workspaces(State(state): State<Arc<AppState>>, auth: AuthUser) -> Json<Vec<DbWorkspace>> {
    let db = state.db.lock().await;
    Json(db.list_workspaces(&auth.user_id))
}

async fn create_workspace(State(state): State<Arc<AppState>>, auth: AuthUser, Json(req): Json<CreateWorkspaceReq>) -> Result<Json<DbWorkspace>, (StatusCode, String)> {
    let ws = DbWorkspace { id: new_id(), name: req.name, description: req.description.unwrap_or_default(), color: req.color, user_id: auth.user_id, created_at: now(), updated_at: now() };
    let db = state.db.lock().await;
    db.create_workspace(&ws).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(ws))
}

async fn delete_workspace(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.db.lock().await;
    db.delete_workspace(&id, &auth.user_id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Project handlers ---

async fn list_projects(State(state): State<Arc<AppState>>, auth: AuthUser) -> Json<Vec<DbProject>> {
    let db = state.db.lock().await;
    Json(db.list_projects(&auth.user_id))
}

async fn create_project(State(state): State<Arc<AppState>>, auth: AuthUser, Json(req): Json<CreateProjectReq>) -> Result<Json<DbProject>, (StatusCode, String)> {
    let p = DbProject {
        id: req.id.unwrap_or_else(new_id), name: req.name, user_id: auth.user_id, workspace_id: req.workspace_id,
        blocks_json: req.blocks_json, variables_json: req.variables_json.unwrap_or_else(|| "{}".into()),
        framework: req.framework, tags_json: req.tags_json.unwrap_or_else(|| "[]".into()), created_at: now(), updated_at: now(),
    };
    let db = state.db.lock().await;
    db.create_project(&p).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(p))
}

async fn update_project(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<String>, Json(req): Json<UpdateProjectReq>) -> Result<Json<DbProject>, (StatusCode, String)> {
    let db = state.db.lock().await;
    let existing = db.list_projects(&auth.user_id).into_iter().find(|p| p.id == id)
        .ok_or((StatusCode::NOT_FOUND, "Project not found".into()))?;
    let updated = DbProject {
        id: existing.id.clone(), name: req.name.unwrap_or(existing.name), user_id: existing.user_id,
        workspace_id: req.workspace_id.or(existing.workspace_id),
        blocks_json: req.blocks_json.unwrap_or(existing.blocks_json),
        variables_json: req.variables_json.unwrap_or(existing.variables_json),
        framework: req.framework.or(existing.framework),
        tags_json: req.tags_json.unwrap_or(existing.tags_json),
        created_at: existing.created_at, updated_at: now(),
    };
    db.update_project(&id, &auth.user_id, &updated).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(updated))
}

async fn delete_project(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.db.lock().await;
    db.delete_project(&id, &auth.user_id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Version handlers ---

async fn list_versions(State(state): State<Arc<AppState>>, auth: AuthUser, Path(project_id): Path<String>) -> Result<Json<Vec<DbVersion>>, (StatusCode, String)> {
    let db = state.db.lock().await;
    if !db.project_belongs_to_user(&project_id, &auth.user_id) { return Err((StatusCode::FORBIDDEN, "Not your project".into())); }
    Ok(Json(db.list_versions(&project_id)))
}

async fn create_version(State(state): State<Arc<AppState>>, auth: AuthUser, Path(project_id): Path<String>, Json(req): Json<CreateVersionReq>) -> Result<Json<DbVersion>, (StatusCode, String)> {
    let db = state.db.lock().await;
    if !db.project_belongs_to_user(&project_id, &auth.user_id) { return Err((StatusCode::FORBIDDEN, "Not your project".into())); }
    let v = DbVersion { id: new_id(), project_id, blocks_json: req.blocks_json, variables_json: req.variables_json, label: req.label, created_at: now() };
    db.create_version(&v).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(v))
}

// --- Execution handlers ---

async fn list_executions(State(state): State<Arc<AppState>>, auth: AuthUser, Path(project_id): Path<String>) -> Result<Json<Vec<DbExecution>>, (StatusCode, String)> {
    let db = state.db.lock().await;
    if !db.project_belongs_to_user(&project_id, &auth.user_id) { return Err((StatusCode::FORBIDDEN, "Not your project".into())); }
    Ok(Json(db.list_executions(&project_id)))
}

async fn create_execution(State(state): State<Arc<AppState>>, auth: AuthUser, Path(project_id): Path<String>, Json(req): Json<CreateExecutionReq>) -> Result<Json<DbExecution>, (StatusCode, String)> {
    let db = state.db.lock().await;
    if !db.project_belongs_to_user(&project_id, &auth.user_id) { return Err((StatusCode::FORBIDDEN, "Not your project".into())); }
    let e = DbExecution { id: new_id(), project_id, model: req.model, provider: req.provider, prompt: req.prompt, response: req.response, tokens_in: req.tokens_in, tokens_out: req.tokens_out, cost: req.cost, latency_ms: req.latency_ms, created_at: now() };
    db.create_execution(&e).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(e))
}

// --- Framework handlers ---

async fn list_frameworks(State(state): State<Arc<AppState>>, auth: AuthUser) -> Json<Vec<DbFramework>> {
    let db = state.db.lock().await;
    Json(db.list_frameworks(&auth.user_id))
}

async fn create_framework(State(state): State<Arc<AppState>>, auth: AuthUser, Json(req): Json<CreateFrameworkReq>) -> Result<Json<DbFramework>, (StatusCode, String)> {
    let f = DbFramework { id: new_id(), name: req.name, description: req.description.unwrap_or_default(), blocks_json: req.blocks_json, user_id: auth.user_id, created_at: now(), updated_at: now() };
    let db = state.db.lock().await;
    db.create_framework(&f).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(f))
}

async fn update_framework(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<String>, Json(req): Json<UpdateFrameworkReq>) -> Result<Json<DbFramework>, (StatusCode, String)> {
    let db = state.db.lock().await;
    let existing = db.list_frameworks(&auth.user_id).into_iter().find(|f| f.id == id)
        .ok_or((StatusCode::NOT_FOUND, "Framework not found".into()))?;
    let updated = DbFramework {
        id: existing.id.clone(), name: req.name.unwrap_or(existing.name), description: req.description.unwrap_or(existing.description),
        blocks_json: req.blocks_json.unwrap_or(existing.blocks_json), user_id: existing.user_id, created_at: existing.created_at, updated_at: now(),
    };
    db.update_framework(&id, &auth.user_id, &updated).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(updated))
}

async fn delete_framework(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.db.lock().await;
    db.delete_framework(&id, &auth.user_id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Config handlers ---

async fn get_config(State(state): State<Arc<AppState>>, auth: AuthUser) -> Json<std::collections::HashMap<String, String>> {
    let db = state.db.lock().await;
    Json(db.get_config(&auth.user_id))
}

async fn set_config(State(state): State<Arc<AppState>>, auth: AuthUser, Json(req): Json<ConfigReq>) -> StatusCode {
    let db = state.db.lock().await;
    for (key, value) in &req.config {
        db.set_config(&auth.user_id, key, value);
    }
    StatusCode::OK
}

// --- OAuth handlers ---

async fn oauth_google(State(state): State<Arc<AppState>>, Json(req): Json<OAuthGoogleReq>) -> Result<Json<AuthResp>, (StatusCode, String)> {
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    if google_client_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Google OAuth is not configured".into()));
    }

    // Verify the Google ID token
    let client = reqwest::Client::new();
    let verify_url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", req.token);
    let resp = client.get(&verify_url).send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to verify Google token: {e}")))?;

    if !resp.status().is_success() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid Google token".into()));
    }

    let info: serde_json::Value = resp.json().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse Google response: {e}")))?;

    // Verify audience matches our client ID
    let aud = info.get("aud").and_then(|v| v.as_str()).unwrap_or_default();
    if aud != google_client_id {
        return Err((StatusCode::UNAUTHORIZED, "Token audience mismatch".into()));
    }

    let email = info.get("email").and_then(|v| v.as_str())
        .ok_or((StatusCode::BAD_REQUEST, "No email in Google token".into()))?
        .to_lowercase();
    let name = info.get("name").and_then(|v| v.as_str()).unwrap_or(&email).to_string();

    // Find or create user
    let db = state.db.lock().await;
    let user = if let Some(existing) = db.get_user_by_email(&email) {
        existing
    } else {
        let random_pw = uuid::Uuid::new_v4().to_string();
        let hash = hash_pw(&random_pw).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        let new_user = DbUser {
            id: new_id(), email: email.clone(), display_name: name.clone(),
            password_hash: hash, avatar: avatar_from_name(&name), created_at: now(),
        };
        db.create_user(&new_user).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        new_user
    };

    let token = create_token(&user.id, &user.email).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(AuthResp { token, user: UserResp { id: user.id, email: user.email, display_name: user.display_name, avatar: user.avatar } }))
}

async fn oauth_github(State(state): State<Arc<AppState>>, Json(req): Json<OAuthGithubReq>) -> Result<Json<AuthResp>, (StatusCode, String)> {
    let github_client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    let github_client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
    if github_client_id.is_empty() || github_client_secret.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "GitHub OAuth is not configured".into()));
    }

    // Exchange code for access token
    let client = reqwest::Client::new();
    let token_resp = client.post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": github_client_id,
            "client_secret": github_client_secret,
            "code": req.code,
        }))
        .send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to exchange GitHub code: {e}")))?;

    let token_data: serde_json::Value = token_resp.json().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse GitHub token response: {e}")))?;

    let access_token = token_data.get("access_token").and_then(|v| v.as_str())
        .ok_or((StatusCode::UNAUTHORIZED, format!("GitHub OAuth failed: {}", token_data.get("error_description").and_then(|v| v.as_str()).unwrap_or("no access token"))))?;

    // Get user info from GitHub
    let user_resp = client.get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {access_token}"))
        .header("User-Agent", "Inkwell-Server")
        .send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to get GitHub user: {e}")))?;

    let user_data: serde_json::Value = user_resp.json().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse GitHub user: {e}")))?;

    let name = user_data.get("name").and_then(|v| v.as_str())
        .or_else(|| user_data.get("login").and_then(|v| v.as_str()))
        .unwrap_or("GitHub User").to_string();

    // Try to get email — may need separate call if private
    let mut email = user_data.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if email.is_empty() {
        let emails_resp = client.get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {access_token}"))
            .header("User-Agent", "Inkwell-Server")
            .send().await.ok();
        if let Some(resp) = emails_resp {
            if let Ok(emails) = resp.json::<Vec<serde_json::Value>>().await {
                email = emails.iter()
                    .find(|e| e.get("primary").and_then(|v| v.as_bool()).unwrap_or(false))
                    .or(emails.first())
                    .and_then(|e| e.get("email").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();
            }
        }
    }

    if email.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Could not retrieve email from GitHub. Make sure your email is public or grant email scope.".into()));
    }

    let email = email.to_lowercase();

    // Find or create user
    let db = state.db.lock().await;
    let user = if let Some(existing) = db.get_user_by_email(&email) {
        existing
    } else {
        let random_pw = uuid::Uuid::new_v4().to_string();
        let hash = hash_pw(&random_pw).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        let new_user = DbUser {
            id: new_id(), email: email.clone(), display_name: name.clone(),
            password_hash: hash, avatar: avatar_from_name(&name), created_at: now(),
        };
        db.create_user(&new_user).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        new_user
    };

    let token = create_token(&user.id, &user.email).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(AuthResp { token, user: UserResp { id: user.id, email: user.email, display_name: user.display_name, avatar: user.avatar } }))
}

// --- Presence handlers ---

async fn set_presence(State(state): State<Arc<AppState>>, auth: AuthUser, Json(req): Json<PresenceReq>) -> Result<StatusCode, (StatusCode, String)> {
    let db = state.db.lock().await;
    let user = db.get_user_by_id(&auth.user_id).ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;
    db.set_presence(&auth.user_id, &req.project_id, &user.display_name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::OK)
}

async fn get_presence(State(state): State<Arc<AppState>>, _auth: AuthUser, Path(project_id): Path<String>) -> Json<Vec<PresenceUser>> {
    let db = state.db.lock().await;
    let since = chrono::Utc::now().timestamp_millis() - 30_000; // last 30 seconds
    let users = db.get_presence(&project_id, since);
    Json(users.into_iter().map(|(user_id, display_name)| PresenceUser { user_id, display_name }).collect())
}
