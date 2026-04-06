use rusqlite::{Connection, params};
use std::path::PathBuf;
use crate::models::*;
use crate::services::auth::{User, hash_password, generate_salt};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open() -> Result<Self, String> {
        let path = db_path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let conn = Connection::open(&path).map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), String> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                display_name TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                user_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                user_id TEXT NOT NULL,
                workspace_id TEXT,
                blocks_json TEXT NOT NULL,
                variables_json TEXT NOT NULL,
                framework TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS versions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                blocks_json TEXT NOT NULL,
                variables_json TEXT NOT NULL,
                label TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS executions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                model TEXT NOT NULL,
                provider TEXT NOT NULL,
                prompt TEXT NOT NULL,
                response TEXT NOT NULL,
                tokens_in INTEGER NOT NULL,
                tokens_out INTEGER NOT NULL,
                cost REAL NOT NULL,
                latency_ms INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        ").map_err(|e| e.to_string())
    }

    // --- Auth ---

    pub fn register(&self, email: &str, password: &str, display_name: &str) -> Result<User, String> {
        let existing: Option<String> = self.conn
            .query_row("SELECT id FROM users WHERE email = ?1", params![email.to_lowercase()], |r| r.get(0))
            .ok();
        if existing.is_some() {
            return Err("EMAIL_EXISTS".into());
        }
        let salt = generate_salt();
        let hash = hash_password(password, &salt);
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.to_lowercase(),
            display_name: display_name.to_string(),
            password_hash: hash,
            salt,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        self.conn.execute(
            "INSERT INTO users (id, email, display_name, password_hash, salt, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![user.id, user.email, user.display_name, user.password_hash, user.salt, user.created_at],
        ).map_err(|e| e.to_string())?;
        Ok(user)
    }

    pub fn login(&self, email: &str, password: &str) -> Result<User, String> {
        let user: User = self.conn.query_row(
            "SELECT id, email, display_name, password_hash, salt, created_at FROM users WHERE email = ?1",
            params![email.to_lowercase()],
            |r| Ok(User {
                id: r.get(0)?, email: r.get(1)?, display_name: r.get(2)?,
                password_hash: r.get(3)?, salt: r.get(4)?, created_at: r.get(5)?,
            }),
        ).map_err(|_| "INVALID_CREDENTIALS".to_string())?;

        let hash = hash_password(password, &user.salt);
        if hash != user.password_hash {
            return Err("INVALID_CREDENTIALS".into());
        }
        Ok(user)
    }

    // --- Workspaces ---

    pub fn list_workspaces(&self, user_id: &str) -> Vec<Workspace> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, user_id, created_at, updated_at FROM workspaces WHERE user_id = ?1 ORDER BY updated_at DESC"
        ).unwrap();
        stmt.query_map(params![user_id], |r| Ok(Workspace {
            id: r.get(0)?, name: r.get(1)?, color: r.get(2)?,
            user_id: r.get(3)?, created_at: r.get(4)?, updated_at: r.get(5)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn create_workspace(&self, ws: &Workspace) -> Result<(), String> {
        self.conn.execute(
            "INSERT INTO workspaces (id, name, color, user_id, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![ws.id, ws.name, ws.color, ws.user_id, ws.created_at, ws.updated_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_workspace(&self, id: &str) -> Result<(), String> {
        self.conn.execute("UPDATE projects SET workspace_id = NULL WHERE workspace_id = ?1", params![id]).ok();
        self.conn.execute("DELETE FROM workspaces WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Projects ---

    pub fn list_projects(&self, user_id: &str) -> Vec<PromptProject> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, user_id, workspace_id, blocks_json, variables_json, framework, created_at, updated_at FROM projects WHERE user_id = ?1 ORDER BY updated_at DESC"
        ).unwrap();
        stmt.query_map(params![user_id], |r| {
            let blocks_json: String = r.get(4)?;
            let vars_json: String = r.get(5)?;
            Ok(PromptProject {
                id: r.get(0)?, name: r.get(1)?, user_id: r.get(2)?, workspace_id: r.get(3)?,
                blocks: serde_json::from_str(&blocks_json).unwrap_or_default(),
                variables: serde_json::from_str(&vars_json).unwrap_or_default(),
                framework: r.get(6)?, created_at: r.get(7)?, updated_at: r.get(8)?,
            })
        }).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn save_project(&self, p: &PromptProject) -> Result<(), String> {
        let blocks_json = serde_json::to_string(&p.blocks).unwrap();
        let vars_json = serde_json::to_string(&p.variables).unwrap();
        self.conn.execute(
            "INSERT OR REPLACE INTO projects (id, name, user_id, workspace_id, blocks_json, variables_json, framework, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![p.id, p.name, p.user_id, p.workspace_id, blocks_json, vars_json, p.framework, p.created_at, p.updated_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM projects WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        self.conn.execute("DELETE FROM versions WHERE project_id = ?1", params![id]).ok();
        self.conn.execute("DELETE FROM executions WHERE project_id = ?1", params![id]).ok();
        Ok(())
    }

    // --- Versions ---

    pub fn list_versions(&self, project_id: &str) -> Vec<PromptVersion> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, blocks_json, variables_json, label, created_at FROM versions WHERE project_id = ?1 ORDER BY created_at DESC"
        ).unwrap();
        stmt.query_map(params![project_id], |r| Ok(PromptVersion {
            id: r.get(0)?, project_id: r.get(1)?, blocks_json: r.get(2)?,
            variables_json: r.get(3)?, label: r.get(4)?, created_at: r.get(5)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    pub fn save_version(&self, v: &PromptVersion) -> Result<(), String> {
        self.conn.execute(
            "INSERT INTO versions (id, project_id, blocks_json, variables_json, label, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![v.id, v.project_id, v.blocks_json, v.variables_json, v.label, v.created_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- Executions ---

    pub fn save_execution(&self, e: &ExecutionResult) -> Result<(), String> {
        self.conn.execute(
            "INSERT INTO executions (id, project_id, model, provider, prompt, response, tokens_in, tokens_out, cost, latency_ms, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![e.id, e.project_id, e.model, e.provider, e.prompt, e.response, e.tokens_in, e.tokens_out, e.cost, e.latency_ms, e.created_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_executions(&self, project_id: &str) -> Vec<ExecutionResult> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, model, provider, prompt, response, tokens_in, tokens_out, cost, latency_ms, created_at FROM executions WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 50"
        ).unwrap();
        stmt.query_map(params![project_id], |r| Ok(ExecutionResult {
            id: r.get(0)?, project_id: r.get(1)?, model: r.get(2)?, provider: r.get(3)?,
            prompt: r.get(4)?, response: r.get(5)?, tokens_in: r.get(6)?, tokens_out: r.get(7)?,
            cost: r.get(8)?, latency_ms: r.get(9)?, created_at: r.get(10)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    }

    // --- Config ---

    pub fn get_config(&self, key: &str) -> Option<String> {
        self.conn.query_row("SELECT value FROM config WHERE key = ?1", params![key], |r| r.get(0)).ok()
    }

    pub fn set_config(&self, key: &str, value: &str) {
        self.conn.execute("INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)", params![key, value]).ok();
    }

    pub fn load_app_config(&self) -> AppConfig {
        let json = self.get_config("app_config").unwrap_or_default();
        serde_json::from_str(&json).unwrap_or_default()
    }

    pub fn save_app_config(&self, config: &AppConfig) {
        let json = serde_json::to_string(config).unwrap();
        self.set_config("app_config", &json);
    }
}

fn db_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("prompt-ide-desktop").join("data.db")
}
