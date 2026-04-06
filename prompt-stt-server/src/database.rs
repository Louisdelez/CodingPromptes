use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub avatar: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbWorkspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbProject {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub blocks_json: String,
    pub variables_json: String,
    pub framework: Option<String>,
    pub tags_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbVersion {
    pub id: String,
    pub project_id: String,
    pub blocks_json: String,
    pub variables_json: String,
    pub label: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbExecution {
    pub id: String,
    pub project_id: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub response: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost: f64,
    pub latency_ms: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbFramework {
    pub id: String,
    pub name: String,
    pub description: String,
    pub blocks_json: String,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Database {
    pub fn open() -> Result<Self, String> {
        let path = db_path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let conn = Connection::open(&path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").ok();
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), String> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY, email TEXT UNIQUE NOT NULL, display_name TEXT NOT NULL,
                password_hash TEXT NOT NULL, avatar TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                color TEXT NOT NULL, user_id TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, user_id TEXT NOT NULL,
                workspace_id TEXT, blocks_json TEXT NOT NULL, variables_json TEXT NOT NULL DEFAULT '{}',
                framework TEXT, tags_json TEXT NOT NULL DEFAULT '[]', created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS versions (
                id TEXT PRIMARY KEY, project_id TEXT NOT NULL, blocks_json TEXT NOT NULL,
                variables_json TEXT NOT NULL DEFAULT '{}', label TEXT NOT NULL, created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS executions (
                id TEXT PRIMARY KEY, project_id TEXT NOT NULL, model TEXT NOT NULL,
                provider TEXT NOT NULL, prompt TEXT NOT NULL, response TEXT NOT NULL,
                tokens_in INTEGER NOT NULL, tokens_out INTEGER NOT NULL, cost REAL NOT NULL,
                latency_ms INTEGER NOT NULL, created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS frameworks (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                blocks_json TEXT NOT NULL, user_id TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS config (
                user_id TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL,
                PRIMARY KEY (user_id, key)
            );
        ").map_err(|e| e.to_string())
    }

    // --- Users ---
    pub fn create_user(&self, user: &DbUser) -> Result<(), String> {
        self.conn.execute(
            "INSERT INTO users (id,email,display_name,password_hash,avatar,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![user.id, user.email, user.display_name, user.password_hash, user.avatar, user.created_at],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn get_user_by_email(&self, email: &str) -> Option<DbUser> {
        self.conn.query_row(
            "SELECT id,email,display_name,password_hash,avatar,created_at FROM users WHERE email=?1",
            params![email], |r| Ok(DbUser { id: r.get(0)?, email: r.get(1)?, display_name: r.get(2)?,
                password_hash: r.get(3)?, avatar: r.get(4)?, created_at: r.get(5)? })).ok()
    }
    pub fn get_user_by_id(&self, id: &str) -> Option<DbUser> {
        self.conn.query_row(
            "SELECT id,email,display_name,password_hash,avatar,created_at FROM users WHERE id=?1",
            params![id], |r| Ok(DbUser { id: r.get(0)?, email: r.get(1)?, display_name: r.get(2)?,
                password_hash: r.get(3)?, avatar: r.get(4)?, created_at: r.get(5)? })).ok()
    }

    // --- Workspaces ---
    pub fn list_workspaces(&self, user_id: &str) -> Vec<DbWorkspace> {
        let mut s = self.conn.prepare("SELECT id,name,description,color,user_id,created_at,updated_at FROM workspaces WHERE user_id=?1 ORDER BY updated_at DESC").unwrap();
        s.query_map(params![user_id], |r| Ok(DbWorkspace { id:r.get(0)?, name:r.get(1)?, description:r.get(2)?, color:r.get(3)?, user_id:r.get(4)?, created_at:r.get(5)?, updated_at:r.get(6)? })).unwrap().filter_map(|r| r.ok()).collect()
    }
    pub fn create_workspace(&self, ws: &DbWorkspace) -> Result<(), String> {
        self.conn.execute("INSERT INTO workspaces (id,name,description,color,user_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![ws.id,ws.name,ws.description,ws.color,ws.user_id,ws.created_at,ws.updated_at]).map_err(|e| e.to_string())?; Ok(())
    }
    pub fn delete_workspace(&self, id: &str, user_id: &str) -> Result<(), String> {
        self.conn.execute("UPDATE projects SET workspace_id=NULL WHERE workspace_id=?1 AND user_id=?2", params![id,user_id]).ok();
        self.conn.execute("DELETE FROM workspaces WHERE id=?1 AND user_id=?2", params![id,user_id]).map_err(|e| e.to_string())?; Ok(())
    }

    // --- Projects ---
    pub fn list_projects(&self, user_id: &str) -> Vec<DbProject> {
        let mut s = self.conn.prepare("SELECT id,name,user_id,workspace_id,blocks_json,variables_json,framework,tags_json,created_at,updated_at FROM projects WHERE user_id=?1 ORDER BY updated_at DESC").unwrap();
        s.query_map(params![user_id], |r| Ok(DbProject { id:r.get(0)?, name:r.get(1)?, user_id:r.get(2)?, workspace_id:r.get(3)?, blocks_json:r.get(4)?, variables_json:r.get(5)?, framework:r.get(6)?, tags_json:r.get(7)?, created_at:r.get(8)?, updated_at:r.get(9)? })).unwrap().filter_map(|r| r.ok()).collect()
    }
    pub fn create_project(&self, p: &DbProject) -> Result<(), String> {
        self.conn.execute("INSERT INTO projects (id,name,user_id,workspace_id,blocks_json,variables_json,framework,tags_json,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![p.id,p.name,p.user_id,p.workspace_id,p.blocks_json,p.variables_json,p.framework,p.tags_json,p.created_at,p.updated_at]).map_err(|e| e.to_string())?; Ok(())
    }
    pub fn update_project(&self, id: &str, user_id: &str, p: &DbProject) -> Result<(), String> {
        self.conn.execute("UPDATE projects SET name=?1,workspace_id=?2,blocks_json=?3,variables_json=?4,framework=?5,tags_json=?6,updated_at=?7 WHERE id=?8 AND user_id=?9",
            params![p.name,p.workspace_id,p.blocks_json,p.variables_json,p.framework,p.tags_json,p.updated_at,id,user_id]).map_err(|e| e.to_string())?; Ok(())
    }
    pub fn delete_project(&self, id: &str, user_id: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM versions WHERE project_id=?1", params![id]).ok();
        self.conn.execute("DELETE FROM executions WHERE project_id=?1", params![id]).ok();
        self.conn.execute("DELETE FROM projects WHERE id=?1 AND user_id=?2", params![id,user_id]).map_err(|e| e.to_string())?; Ok(())
    }

    // --- Versions ---
    pub fn list_versions(&self, project_id: &str) -> Vec<DbVersion> {
        let mut s = self.conn.prepare("SELECT id,project_id,blocks_json,variables_json,label,created_at FROM versions WHERE project_id=?1 ORDER BY created_at DESC").unwrap();
        s.query_map(params![project_id], |r| Ok(DbVersion { id:r.get(0)?, project_id:r.get(1)?, blocks_json:r.get(2)?, variables_json:r.get(3)?, label:r.get(4)?, created_at:r.get(5)? })).unwrap().filter_map(|r| r.ok()).collect()
    }
    pub fn create_version(&self, v: &DbVersion) -> Result<(), String> {
        self.conn.execute("INSERT INTO versions (id,project_id,blocks_json,variables_json,label,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![v.id,v.project_id,v.blocks_json,v.variables_json,v.label,v.created_at]).map_err(|e| e.to_string())?; Ok(())
    }

    // --- Executions ---
    pub fn list_executions(&self, project_id: &str) -> Vec<DbExecution> {
        let mut s = self.conn.prepare("SELECT id,project_id,model,provider,prompt,response,tokens_in,tokens_out,cost,latency_ms,created_at FROM executions WHERE project_id=?1 ORDER BY created_at DESC LIMIT 100").unwrap();
        s.query_map(params![project_id], |r| Ok(DbExecution { id:r.get(0)?, project_id:r.get(1)?, model:r.get(2)?, provider:r.get(3)?, prompt:r.get(4)?, response:r.get(5)?, tokens_in:r.get(6)?, tokens_out:r.get(7)?, cost:r.get(8)?, latency_ms:r.get(9)?, created_at:r.get(10)? })).unwrap().filter_map(|r| r.ok()).collect()
    }
    pub fn create_execution(&self, e: &DbExecution) -> Result<(), String> {
        self.conn.execute("INSERT INTO executions (id,project_id,model,provider,prompt,response,tokens_in,tokens_out,cost,latency_ms,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![e.id,e.project_id,e.model,e.provider,e.prompt,e.response,e.tokens_in,e.tokens_out,e.cost,e.latency_ms,e.created_at]).map_err(|e| e.to_string())?; Ok(())
    }

    // --- Frameworks ---
    pub fn list_frameworks(&self, user_id: &str) -> Vec<DbFramework> {
        let mut s = self.conn.prepare("SELECT id,name,description,blocks_json,user_id,created_at,updated_at FROM frameworks WHERE user_id=?1 ORDER BY updated_at DESC").unwrap();
        s.query_map(params![user_id], |r| Ok(DbFramework { id:r.get(0)?, name:r.get(1)?, description:r.get(2)?, blocks_json:r.get(3)?, user_id:r.get(4)?, created_at:r.get(5)?, updated_at:r.get(6)? })).unwrap().filter_map(|r| r.ok()).collect()
    }
    pub fn create_framework(&self, f: &DbFramework) -> Result<(), String> {
        self.conn.execute("INSERT INTO frameworks (id,name,description,blocks_json,user_id,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![f.id,f.name,f.description,f.blocks_json,f.user_id,f.created_at,f.updated_at]).map_err(|e| e.to_string())?; Ok(())
    }
    pub fn update_framework(&self, id: &str, user_id: &str, f: &DbFramework) -> Result<(), String> {
        self.conn.execute("UPDATE frameworks SET name=?1,description=?2,blocks_json=?3,updated_at=?4 WHERE id=?5 AND user_id=?6",
            params![f.name,f.description,f.blocks_json,f.updated_at,id,user_id]).map_err(|e| e.to_string())?; Ok(())
    }
    pub fn delete_framework(&self, id: &str, user_id: &str) -> Result<(), String> {
        self.conn.execute("DELETE FROM frameworks WHERE id=?1 AND user_id=?2", params![id,user_id]).map_err(|e| e.to_string())?; Ok(())
    }

    // --- Config ---
    pub fn get_config(&self, user_id: &str) -> std::collections::HashMap<String, String> {
        let mut s = self.conn.prepare("SELECT key,value FROM config WHERE user_id=?1").unwrap();
        s.query_map(params![user_id], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?)))
            .unwrap().filter_map(|r| r.ok()).collect()
    }
    pub fn set_config(&self, user_id: &str, key: &str, value: &str) {
        self.conn.execute("INSERT OR REPLACE INTO config (user_id,key,value) VALUES (?1,?2,?3)", params![user_id,key,value]).ok();
    }

    // --- Helpers ---
    pub fn project_belongs_to_user(&self, project_id: &str, user_id: &str) -> bool {
        self.conn.query_row("SELECT 1 FROM projects WHERE id=?1 AND user_id=?2", params![project_id,user_id], |_| Ok(())).is_ok()
    }
}

fn db_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("prompt-ai-server").join("data.db")
}
