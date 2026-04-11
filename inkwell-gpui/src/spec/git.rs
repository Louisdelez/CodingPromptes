//! Git integration for Spec-Driven Development.
//! Creates feature branches, auto-commits spec artifacts.

use std::process::Command;
use std::path::Path;

/// Check if current directory is a git repo
pub fn is_git_repo(dir: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get current branch name
pub fn current_branch(dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(dir)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else { None }
        })
}

/// Create and checkout a feature branch
pub fn create_feature_branch(dir: &Path, feature_num: u32, feature_name: &str) -> Result<String, String> {
    let branch_name = format!("{:03}-{}", feature_num, feature_name.to_lowercase().replace(' ', "-"));

    let output = Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if output.status.success() {
        Ok(branch_name)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Auto-commit spec artifacts
pub fn commit_specs(dir: &Path, message: &str) -> Result<(), String> {
    // Stage native .inkwell/ directory
    let _ = Command::new("git").args(["add", ".inkwell/"]).current_dir(dir).output();

    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if output.status.success() { Ok(()) }
    else { Err(String::from_utf8_lossy(&output.stderr).trim().to_string()) }
}

/// List recent commits (for history)
pub fn recent_commits(dir: &Path, count: usize) -> Vec<(String, String)> {
    Command::new("git")
        .args(["log", &format!("--max-count={}", count), "--oneline"])
        .current_dir(dir)
        .output()
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| {
                    let parts: Vec<&str> = l.splitn(2, ' ').collect();
                    (parts.first().unwrap_or(&"").to_string(), parts.get(1).unwrap_or(&"").to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}
