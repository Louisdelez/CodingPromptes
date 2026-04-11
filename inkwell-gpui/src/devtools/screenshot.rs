use serde_json::json;

/// Find the X11 window that belongs to the current inkwell-gpui process.
/// Filters by PID so a stray legacy binary (prompt-ai-server) can't steal the screenshot.
async fn find_own_window() -> Option<String> {
    let my_pid = std::process::id().to_string();

    // Strategy 1: xdotool search --pid (exact process match)
    if let Ok(output) = tokio::process::Command::new("xdotool")
        .args(["search", "--pid", &my_pid])
        .output().await
    {
        let ids = String::from_utf8_lossy(&output.stdout);
        // Prefer a window whose name is exactly "Inkwell" (filters out child menus/tooltips)
        for id in ids.lines().filter(|s| !s.is_empty()) {
            if let Ok(name_out) = tokio::process::Command::new("xdotool")
                .args(["getwindowname", id])
                .output().await
            {
                let name = String::from_utf8_lossy(&name_out.stdout);
                let trimmed = name.trim();
                if trimmed == "Inkwell" || trimmed.starts_with("Inkwell ") {
                    return Some(id.to_string());
                }
            }
        }
        // Fallback to first window of this PID
        if let Some(first) = ids.lines().next().filter(|s| !s.is_empty()) {
            return Some(first.to_string());
        }
    }

    // Strategy 2: fall back to name search + PID verification
    if let Ok(output) = tokio::process::Command::new("xdotool")
        .args(["search", "--name", "^Inkwell$"])
        .output().await
    {
        let ids = String::from_utf8_lossy(&output.stdout);
        for id in ids.lines().filter(|s| !s.is_empty()) {
            if let Ok(pid_out) = tokio::process::Command::new("xdotool")
                .args(["getwindowpid", id])
                .output().await
            {
                let pid_str = String::from_utf8_lossy(&pid_out.stdout).trim().to_string();
                if pid_str == my_pid {
                    return Some(id.to_string());
                }
            }
        }
    }

    None
}

pub async fn capture() -> serde_json::Value {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let path = format!("/tmp/inkwell-screenshot-{}.png", timestamp);

    // xdotool + import (ImageMagick), filtered by our own PID
    if let Some(wid) = find_own_window().await {
        if let Ok(status) = tokio::process::Command::new("import")
            .args(["-window", &wid, &path])
            .status().await
        {
            if status.success() {
                return json!({"ok": true, "path": path, "window_id": wid});
            }
        }
    }

    // Fallback: scrot with focused window
    if let Ok(status) = tokio::process::Command::new("scrot")
        .args(["-u", &path])
        .status().await
    {
        if status.success() {
            return json!({"ok": true, "path": path, "via": "scrot"});
        }
    }

    // Fallback: gnome-screenshot
    if let Ok(status) = tokio::process::Command::new("gnome-screenshot")
        .args(["-w", "-f", &path])
        .status().await
    {
        if status.success() {
            return json!({"ok": true, "path": path, "via": "gnome-screenshot"});
        }
    }

    json!({"ok": false, "error": "No screenshot tool available (tried xdotool+import, scrot, gnome-screenshot)"})
}
