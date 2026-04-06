use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub url: String,
    pub enabled: bool,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:11434".into(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub connected: bool,
    pub models: Vec<OllamaModel>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Clone)]
pub struct OllamaState {
    pub config: Arc<RwLock<OllamaConfig>>,
    pub status: Arc<RwLock<OllamaStatus>>,
    pub http_client: reqwest::Client,
}

impl OllamaState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(OllamaConfig::default())),
            status: Arc::new(RwLock::new(OllamaStatus {
                connected: false,
                models: vec![],
                error: None,
            })),
            http_client: reqwest::Client::new(),
        }
    }

    /// Check if Ollama is reachable and list models
    pub async fn refresh_status(&self) {
        let config = self.config.read().await.clone();
        if !config.enabled {
            let mut s = self.status.write().await;
            s.connected = false;
            s.models.clear();
            s.error = Some("Ollama desactive".into());
            return;
        }

        let url = format!("{}/api/tags", config.url.trim_end_matches('/'));
        match self.http_client.get(&url).timeout(std::time::Duration::from_secs(3)).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let models: Vec<OllamaModel> = body["models"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|m| OllamaModel {
                            name: m["name"].as_str().unwrap_or("").to_string(),
                            size: m["size"].as_u64().unwrap_or(0),
                            parameter_size: m["details"]["parameter_size"].as_str().map(String::from),
                            quantization_level: m["details"]["quantization_level"].as_str().map(String::from),
                        })
                        .collect();

                    let mut s = self.status.write().await;
                    s.connected = true;
                    s.models = models;
                    s.error = None;
                }
            }
            Ok(resp) => {
                let mut s = self.status.write().await;
                s.connected = false;
                s.models.clear();
                s.error = Some(format!("Ollama HTTP {}", resp.status()));
            }
            Err(e) => {
                let mut s = self.status.write().await;
                s.connected = false;
                s.models.clear();
                s.error = Some(format!("Connexion impossible: {e}"));
            }
        }
    }

    /// Proxy a request body to Ollama and return the raw response
    pub async fn proxy_chat(&self, body: bytes::Bytes) -> Result<reqwest::Response, String> {
        let config = self.config.read().await;
        if !config.enabled {
            return Err("Ollama desactive".into());
        }
        let url = format!("{}/v1/chat/completions", config.url.trim_end_matches('/'));
        drop(config);

        self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("Proxy Ollama error: {e}"))
    }

    /// Proxy models list
    pub async fn proxy_models(&self) -> Result<reqwest::Response, String> {
        let config = self.config.read().await;
        let url = format!("{}/v1/models", config.url.trim_end_matches('/'));
        drop(config);

        self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Proxy Ollama error: {e}"))
    }
}
