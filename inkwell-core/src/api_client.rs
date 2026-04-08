use crate::types::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct LoginReq { email: String, password: String }

#[derive(Serialize)]
struct RegisterReq { email: String, password: String, display_name: String }

#[derive(Deserialize)]
struct AuthResp {
    token: String,
    user: UserResp,
}

#[derive(Deserialize)]
struct UserResp {
    id: String,
    email: String,
    display_name: String,
    avatar: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    // --- Auth ---

    pub async fn login(&mut self, email: &str, password: &str) -> Result<AuthSession, String> {
        let resp = self.client.post(self.url("/auth/login"))
            .json(&LoginReq { email: email.into(), password: password.into() })
            .send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(resp.text().await.unwrap_or_default());
        }

        let data: AuthResp = resp.json().await.map_err(|e| e.to_string())?;
        self.token = Some(data.token.clone());

        Ok(AuthSession {
            user_id: data.user.id,
            email: data.user.email,
            display_name: data.user.display_name,
            avatar: data.user.avatar,
            token: data.token,
        })
    }

    pub async fn register(&mut self, email: &str, password: &str, display_name: &str) -> Result<AuthSession, String> {
        let resp = self.client.post(self.url("/auth/register"))
            .json(&RegisterReq { email: email.into(), password: password.into(), display_name: display_name.into() })
            .send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(resp.text().await.unwrap_or_default());
        }

        let data: AuthResp = resp.json().await.map_err(|e| e.to_string())?;
        self.token = Some(data.token.clone());

        Ok(AuthSession {
            user_id: data.user.id,
            email: data.user.email,
            display_name: data.user.display_name,
            avatar: data.user.avatar,
            token: data.token,
        })
    }

    pub async fn get_me(&self) -> Result<AuthSession, String> {
        let mut req = self.client.get(self.url("/auth/me"));
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err("UNAUTHORIZED".into());
        }
        let user: UserResp = resp.json().await.map_err(|e| e.to_string())?;
        Ok(AuthSession {
            user_id: user.id,
            email: user.email,
            display_name: user.display_name,
            avatar: user.avatar,
            token: self.token.clone().unwrap_or_default(),
        })
    }

    // --- Projects ---

    pub async fn list_projects(&self) -> Result<Vec<PromptProject>, String> {
        let mut req = self.client.get(self.url("/projects"));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn create_project(&self, project: &serde_json::Value) -> Result<PromptProject, String> {
        let mut req = self.client.post(self.url("/projects")).json(project);
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        resp.json().await.map_err(|e| e.to_string())
    }

    pub async fn update_project(&self, id: &str, update: &serde_json::Value) -> Result<(), String> {
        let mut req = self.client.put(self.url(&format!("/projects/{id}"))).json(update);
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        Ok(())
    }

    pub async fn delete_project(&self, id: &str) -> Result<(), String> {
        let mut req = self.client.delete(self.url(&format!("/projects/{id}")));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        Ok(())
    }

    // --- Workspaces ---

    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>, String> {
        let mut req = self.client.get(self.url("/workspaces"));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        resp.json().await.map_err(|e| e.to_string())
    }

    // --- GPU Nodes ---

    pub async fn list_nodes(&self) -> Result<Vec<GpuNode>, String> {
        let mut req = self.client.get(self.url("/nodes"));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        resp.json().await.map_err(|e| e.to_string())
    }

    // --- Versions ---

    pub async fn create_version(&self, project_id: &str, blocks_json: &str, variables_json: &str, label: &str) -> Result<(), String> {
        let mut req = self.client.post(self.url(&format!("/projects/{project_id}/versions")))
            .json(&serde_json::json!({ "blocks_json": blocks_json, "variables_json": variables_json, "label": label }));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        Ok(())
    }

    pub async fn list_versions(&self, project_id: &str) -> Result<Vec<Version>, String> {
        let mut req = self.client.get(self.url(&format!("/projects/{project_id}/versions")));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        resp.json().await.map_err(|e| e.to_string())
    }

    // --- Executions ---

    pub async fn create_execution(&self, project_id: &str, data: &serde_json::Value) -> Result<(), String> {
        let mut req = self.client.post(self.url(&format!("/projects/{project_id}/executions"))).json(data);
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        Ok(())
    }

    pub async fn list_executions(&self, project_id: &str) -> Result<Vec<ExecutionResult>, String> {
        let mut req = self.client.get(self.url(&format!("/projects/{project_id}/executions")));
        if let Some(auth) = self.auth_header() { req = req.header("Authorization", auth); }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() { return Err(resp.text().await.unwrap_or_default()); }
        resp.json().await.map_err(|e| e.to_string())
    }
}
