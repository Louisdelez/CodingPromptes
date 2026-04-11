use std::io::{self, BufRead, Write};
use serde::Deserialize;

mod tools;
mod socket;

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() { continue; }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                let err = serde_json::json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}});
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", err);
                let _ = out.flush();
                continue;
            }
        };

        let id = req.id.clone().unwrap_or(serde_json::Value::Null);

        // Handle notifications (no response)
        if req.method.starts_with("notifications/") {
            continue;
        }

        let response = handle_request(&req.method, &req.params, &id, &rt);

        let mut out = stdout.lock();
        let _ = writeln!(out, "{}", response);
        let _ = out.flush();
    }
}

fn handle_request(method: &str, params: &serde_json::Value, id: &serde_json::Value, rt: &tokio::runtime::Runtime) -> String {
    match method {
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "inkwell-devtools-mcp",
                    "version": "0.1.0"
                }
            });
            serde_json::json!({"jsonrpc":"2.0","id":id,"result":result}).to_string()
        }

        "tools/list" => {
            let tools = tools::list_tools();
            serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"tools":tools}}).to_string()
        }

        "tools/call" => {
            let tool_name = params["name"].as_str().unwrap_or("");
            let args = &params["arguments"];
            let result = rt.block_on(tools::call_tool(tool_name, args));
            serde_json::json!({
                "jsonrpc":"2.0","id":id,
                "result":{"content":[{"type":"text","text":result}]}
            }).to_string()
        }

        _ => {
            serde_json::json!({
                "jsonrpc":"2.0","id":id,
                "error":{"code":-32601,"message":format!("Method not found: {}", method)}
            }).to_string()
        }
    }
}
