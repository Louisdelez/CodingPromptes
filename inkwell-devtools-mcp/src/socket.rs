use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn socket_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("inkwell")
        .join("devtools.sock")
}

/// Send a JSON-RPC request to the GPUI app's Unix socket and return the result.
pub async fn call(method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let path = socket_path();

    let stream = UnixStream::connect(&path).await
        .map_err(|e| format!("Cannot connect to Inkwell app ({}). Is inkwell-gpui running?", e))?;

    let (reader, mut writer) = stream.into_split();

    let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });

    let msg = format!("{}\n", serde_json::to_string(&request).unwrap_or_default());
    writer.write_all(msg.as_bytes()).await
        .map_err(|e| format!("Write error: {}", e))?;

    let mut lines = BufReader::new(reader).lines();

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        lines.next_line(),
    ).await {
        Ok(Ok(Some(line))) => {
            let resp: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| format!("Invalid response JSON: {}", e))?;
            Ok(resp["result"].clone())
        }
        Ok(Ok(None)) => Err("Connection closed".to_string()),
        Ok(Err(e)) => Err(format!("Read error: {}", e)),
        Err(_) => Err("Response timeout (10s)".to_string()),
    }
}
