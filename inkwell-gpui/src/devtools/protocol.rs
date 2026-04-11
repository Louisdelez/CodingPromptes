use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: serde_json::Value,
    pub result: serde_json::Value,
}

#[derive(Serialize)]
pub struct JsonRpcError {
    pub jsonrpc: &'static str,
    pub id: serde_json::Value,
    pub error: JsonRpcErrorBody,
}

#[derive(Serialize)]
pub struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn ok(id: serde_json::Value, result: serde_json::Value) -> String {
        serde_json::to_string(&Self { jsonrpc: "2.0", id, result }).unwrap_or_default()
    }
}

impl JsonRpcError {
    pub fn new(id: serde_json::Value, code: i32, message: &str) -> String {
        serde_json::to_string(&Self {
            jsonrpc: "2.0", id,
            error: JsonRpcErrorBody { code, message: message.to_string() },
        }).unwrap_or_default()
    }
}
