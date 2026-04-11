use serde_json::json;

pub async fn capture() -> serde_json::Value {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let path = format!("/tmp/inkwell-screenshot-{}.png", timestamp);

    // Try xdotool + import (ImageMagick) first
    if let Ok(output) = tokio::process::Command::new("xdotool")
        .args(["search", "--name", "Inkwell"])
        .output().await
    {
        let ids = String::from_utf8_lossy(&output.stdout);
        if let Some(first_id) = ids.lines().next().filter(|s| !s.is_empty()) {
            if let Ok(status) = tokio::process::Command::new("import")
                .args(["-window", first_id, &path])
                .status().await
            {
                if status.success() {
                    return json!({"ok": true, "path": path});
                }
            }
        }
    }

    // Fallback: scrot with focused window
    if let Ok(status) = tokio::process::Command::new("scrot")
        .args(["-u", &path])
        .status().await
    {
        if status.success() {
            return json!({"ok": true, "path": path});
        }
    }

    // Fallback: gnome-screenshot
    if let Ok(status) = tokio::process::Command::new("gnome-screenshot")
        .args(["-w", "-f", &path])
        .status().await
    {
        if status.success() {
            return json!({"ok": true, "path": path});
        }
    }

    json!({"ok": false, "error": "No screenshot tool available (tried xdotool+import, scrot, gnome-screenshot)"})
}
