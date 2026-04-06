use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BackendClient {
    base_url: String,
    token: Option<String>,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: BackendUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendWorkspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendProject {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub blocks_json: String,
    pub variables_json: String,
    pub framework: Option<String>,
    pub tags_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendVersion {
    pub id: String,
    pub project_id: String,
    pub blocks_json: String,
    pub variables_json: String,
    pub label: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendExecution {
    pub id: String,
    pub project_id: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub response: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost: f64,
    pub latency_ms: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendFramework {
    pub id: String,
    pub name: String,
    pub description: String,
    pub blocks_json: String,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl BackendClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn is_logged_in(&self) -> bool {
        self.token.is_some()
    }

    pub fn set_base_url(&mut self, url: &str) {
        self.base_url = url.trim_end_matches('/').to_string();
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        let mut req = self.client.get(format!("{}/api{path}", self.base_url));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(text);
        }
        res.json().await.map_err(|e| e.to_string())
    }

    async fn post<T: serde::de::DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, String> {
        let mut req = self.client.post(format!("{}/api{path}", self.base_url)).json(body);
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(text);
        }
        res.json().await.map_err(|e| e.to_string())
    }

    async fn put<T: serde::de::DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T, String> {
        let mut req = self.client.put(format!("{}/api{path}", self.base_url)).json(body);
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(text);
        }
        res.json().await.map_err(|e| e.to_string())
    }

    async fn delete_req(&self, path: &str) -> Result<(), String> {
        let mut req = self.client.delete(format!("{}/api{path}", self.base_url));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(text);
        }
        Ok(())
    }

    // --- Auth ---

    pub async fn register(&self, email: &str, password: &str, display_name: &str) -> Result<AuthResponse, String> {
        self.post("/auth/register", &serde_json::json!({ "email": email, "password": password, "display_name": display_name })).await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        self.post("/auth/login", &serde_json::json!({ "email": email, "password": password })).await
    }

    pub async fn get_me(&self) -> Result<BackendUser, String> {
        self.get("/auth/me").await
    }

    // --- Workspaces ---

    pub async fn list_workspaces(&self) -> Result<Vec<BackendWorkspace>, String> {
        self.get("/workspaces").await
    }

    pub async fn create_workspace(&self, name: &str, color: &str) -> Result<BackendWorkspace, String> {
        self.post("/workspaces", &serde_json::json!({ "name": name, "color": color })).await
    }

    pub async fn delete_workspace(&self, id: &str) -> Result<(), String> {
        self.delete_req(&format!("/workspaces/{id}")).await
    }

    // --- Projects ---

    pub async fn list_projects(&self) -> Result<Vec<BackendProject>, String> {
        self.get("/projects").await
    }

    pub async fn create_project(&self, data: &serde_json::Value) -> Result<BackendProject, String> {
        self.post("/projects", data).await
    }

    pub async fn update_project(&self, id: &str, data: &serde_json::Value) -> Result<BackendProject, String> {
        self.put(&format!("/projects/{id}"), data).await
    }

    pub async fn delete_project(&self, id: &str) -> Result<(), String> {
        self.delete_req(&format!("/projects/{id}")).await
    }

    // --- Versions ---

    pub async fn list_versions(&self, project_id: &str) -> Result<Vec<BackendVersion>, String> {
        self.get(&format!("/projects/{project_id}/versions")).await
    }

    pub async fn create_version(&self, project_id: &str, data: &serde_json::Value) -> Result<BackendVersion, String> {
        self.post(&format!("/projects/{project_id}/versions"), data).await
    }

    // --- Executions ---

    pub async fn list_executions(&self, project_id: &str) -> Result<Vec<BackendExecution>, String> {
        self.get(&format!("/projects/{project_id}/executions")).await
    }

    pub async fn create_execution(&self, project_id: &str, data: &serde_json::Value) -> Result<BackendExecution, String> {
        self.post(&format!("/projects/{project_id}/executions"), data).await
    }

    // --- Frameworks ---

    pub async fn list_frameworks(&self) -> Result<Vec<BackendFramework>, String> {
        self.get("/frameworks").await
    }

    pub async fn create_framework(&self, data: &serde_json::Value) -> Result<BackendFramework, String> {
        self.post("/frameworks", data).await
    }

    pub async fn update_framework(&self, id: &str, data: &serde_json::Value) -> Result<BackendFramework, String> {
        self.put(&format!("/frameworks/{id}"), data).await
    }

    pub async fn delete_framework(&self, id: &str) -> Result<(), String> {
        self.delete_req(&format!("/frameworks/{id}")).await
    }

    // --- Config ---

    pub async fn get_config(&self) -> Result<HashMap<String, String>, String> {
        self.get("/config").await
    }

    pub async fn set_config(&self, config: &HashMap<String, String>) -> Result<(), String> {
        let mut req = self.client.put(format!("{}/api/config", self.base_url))
            .json(&serde_json::json!({ "config": config }));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let res = req.send().await.map_err(|e| e.to_string())?;
        if !res.status().is_success() { return Err(res.text().await.unwrap_or_default()); }
        Ok(())
    }
}
