use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ollama::OllamaState;
use crate::whisper_engine::WhisperEngine;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetConfig {
    pub api_url: String,
    pub node_id: String,
    pub node_name: String,
    pub jwt_token: String,
    pub user_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetStatus {
    pub connected: bool,
    pub last_heartbeat: Option<i64>,
    pub error: Option<String>,
}

impl Default for FleetStatus {
    fn default() -> Self {
        Self { connected: false, last_heartbeat: None, error: None }
    }
}

#[derive(Clone)]
pub struct FleetState {
    pub config: Arc<RwLock<FleetConfig>>,
    pub status: Arc<RwLock<FleetStatus>>,
}

impl FleetState {
    pub fn new() -> Self {
        let config = load_config().unwrap_or_default();
        Self {
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(FleetStatus::default())),
        }
    }

    /// Login to the central backend and register this node
    pub async fn login(
        &self,
        api_url: &str,
        email: &str,
        password: &str,
        node_name: &str,
        local_port: u16,
    ) -> Result<String, String> {
        let client = reqwest::Client::new();

        // Login
        let login_resp = client
            .post(format!("{api_url}/api/auth/login"))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        if !login_resp.status().is_success() {
            let text = login_resp.text().await.unwrap_or_default();
            return Err(format!("Login failed: {text}"));
        }

        let login_data: serde_json::Value = login_resp.json().await.map_err(|e| e.to_string())?;
        let token = login_data["token"].as_str().ok_or("No token in response")?.to_string();

        // Get local IP for address
        let local_addr = get_local_address(local_port);

        // Check if we already have a node_id (re-login)
        let mut config = self.config.write().await;
        let node_id = if !config.node_id.is_empty() {
            // Verify node still exists
            let check = client
                .get(format!("{api_url}/api/nodes"))
                .header("Authorization", format!("Bearer {token}"))
                .send()
                .await;

            let exists = if let Ok(resp) = check {
                if let Ok(nodes) = resp.json::<Vec<serde_json::Value>>().await {
                    nodes.iter().any(|n| n["id"].as_str() == Some(&config.node_id))
                } else { false }
            } else { false };

            if exists {
                config.node_id.clone()
            } else {
                register_new_node(&client, api_url, &token, node_name, &local_addr).await?
            }
        } else {
            register_new_node(&client, api_url, &token, node_name, &local_addr).await?
        };

        // Save config
        config.api_url = api_url.to_string();
        config.node_id = node_id.clone();
        config.node_name = node_name.to_string();
        config.jwt_token = token;
        config.user_email = email.to_string();
        save_config(&config);
        drop(config);

        // Update status
        let mut status = self.status.write().await;
        status.connected = true;
        status.error = None;

        Ok(node_id)
    }

    pub async fn disconnect(&self) {
        let mut config = self.config.write().await;
        config.jwt_token.clear();
        save_config(&config);
        drop(config);

        let mut status = self.status.write().await;
        status.connected = false;
    }

    /// Send a heartbeat with current capabilities
    pub async fn send_heartbeat(&self, engine: &WhisperEngine, ollama: &OllamaState, port: u16) -> Result<(), String> {
        let config = self.config.read().await;
        if config.jwt_token.is_empty() || config.node_id.is_empty() {
            return Ok(());
        }

        let api_url = config.api_url.clone();
        let node_id = config.node_id.clone();
        let token = config.jwt_token.clone();
        drop(config);

        // Build capabilities
        let stt_loaded = engine.is_loaded();
        let active_model = engine.current_model_path().and_then(|p| {
            crate::models::available_models().iter()
                .find(|m| p.contains(&m.filename))
                .map(|m| m.id.clone())
        });
        let installed: Vec<String> = crate::models::installed_models().iter().map(|m| m.id.clone()).collect();

        let ollama_status = ollama.status.read().await;
        let ollama_models: Vec<serde_json::Value> = ollama_status.models.iter().map(|m| {
            serde_json::json!({
                "name": m.name,
                "size_gb": m.size as f64 / 1_073_741_824.0,
                "parameter_size": m.parameter_size,
            })
        }).collect();

        let capabilities = serde_json::json!({
            "stt": {
                "model_loaded": stt_loaded,
                "active_model": active_model,
                "available_models": installed,
            },
            "llm": {
                "ollama_connected": ollama_status.connected,
                "models": ollama_models,
            }
        });
        drop(ollama_status);

        let address = get_local_address(port);

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{api_url}/api/nodes/{node_id}/heartbeat"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({
                "status": "online",
                "capabilities": capabilities,
                "address": address,
            }))
            .send()
            .await
            .map_err(|e| format!("Heartbeat failed: {e}"))?;

        if resp.status().as_u16() == 401 {
            return Err("Token expired".into());
        }
        if !resp.status().is_success() {
            return Err(format!("Heartbeat error: {}", resp.status()));
        }

        let mut status = self.status.write().await;
        status.connected = true;
        status.last_heartbeat = Some(chrono::Utc::now().timestamp_millis());
        status.error = None;

        Ok(())
    }

    /// Start the heartbeat background loop
    pub fn start_heartbeat_loop(&self, engine: WhisperEngine, ollama: OllamaState, port: u16) {
        let fleet = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                if let Err(e) = fleet.send_heartbeat(&engine, &ollama, port).await {
                    let mut status = fleet.status.write().await;
                    status.error = Some(e);
                }
            }
        });
    }

    pub async fn is_configured(&self) -> bool {
        let config = self.config.read().await;
        !config.jwt_token.is_empty() && !config.node_id.is_empty()
    }
}

async fn register_new_node(client: &reqwest::Client, api_url: &str, token: &str, name: &str, address: &str) -> Result<String, String> {
    let hostname = hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();
    let gpu_info = detect_gpu_info();

    let resp = client
        .post(format!("{api_url}/api/nodes"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "name": name,
            "address": address,
            "hostname": hostname,
            "gpu_info": gpu_info,
        }))
        .send()
        .await
        .map_err(|e| format!("Registration failed: {e}"))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Registration failed: {text}"));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    data["id"].as_str().map(|s| s.to_string()).ok_or("No node ID in response".into())
}

fn get_local_address(port: u16) -> String {
    // Try to find LAN IP
    if let Ok(addrs) = local_ip_address::local_ip() {
        return format!("http://{}:{}", addrs, port);
    }
    format!("http://0.0.0.0:{port}")
}

fn detect_gpu_info() -> String {
    // Try nvidia-smi
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                // Parse "NVIDIA GeForce RTX 3060, 12288" -> "NVIDIA GeForce RTX 3060 12GB"
                let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
                if parts.len() >= 2 {
                    if let Ok(mb) = parts[1].parse::<u64>() {
                        return format!("{} {}GB", parts[0], mb / 1024);
                    }
                }
                return s;
            }
        }
    }
    "Unknown GPU".into()
}

// --- Config persistence ---

fn config_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("inkwell-server").join("fleet.json")
}

fn load_config() -> Option<FleetConfig> {
    let path = config_path();
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_config(config: &FleetConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        std::fs::write(path, json).ok();
    }
}
