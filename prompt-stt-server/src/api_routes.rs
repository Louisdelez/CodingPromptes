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
use crate::server::AppState;

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

fn now() -> i64 { chrono::Utc::now().timestamp_millis() }
fn new_id() -> String { uuid::Uuid::new_v4().to_string() }

fn hash_pw(password: &str) -> Result<String, String> {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
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
