//! Inkwell MCP Server — Model Context Protocol via stdio
//! Exposes Inkwell SDD tools to AI agents (Claude Code, etc.)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

mod tools;

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    error: Option<Value>,
}

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    eprintln!("Inkwell MCP Server started (stdio)");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() { continue; }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = handle_request(&request);
        let json = serde_json::to_string(&response).unwrap_or_default();
        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", json);
        let _ = out.flush();
    }
}

fn handle_request(req: &JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone().unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".into(), id, error: None,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "inkwell-mcp",
                    "version": "0.1.0"
                }
            })),
        },
        "notifications/initialized" => JsonRpcResponse {
            jsonrpc: "2.0".into(), id, result: Some(json!({})), error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(), id, error: None,
            result: Some(json!({ "tools": tools::list_tools() })),
        },
        "tools/call" => {
            let params = req.params.as_ref();
            let tool_name = params.and_then(|p| p["name"].as_str()).unwrap_or("");
            let args = params.and_then(|p| p.get("arguments")).cloned().unwrap_or(json!({}));
            let result = tools::call_tool(tool_name, &args);
            JsonRpcResponse {
                jsonrpc: "2.0".into(), id, error: None,
                result: Some(json!({
                    "content": [{ "type": "text", "text": result }]
                })),
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".into(), id, result: None,
            error: Some(json!({ "code": -32601, "message": format!("Method not found: {}", req.method) })),
        },
    }
}
