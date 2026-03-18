//! SQLite-based exchange logging for ACP proxy sessions.
//!
//! Logs prompt/response exchanges as they flow through the proxy,
//! enabling auto-checkpoint on next session load.

use crate::sparkle_paths::get_sparkle_space_dir;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;

const MAX_SESSIONS_KEPT: usize = 10;

/// Database handle for exchange logging.
pub struct ExchangeDb {
    conn: Mutex<Connection>,
}

impl ExchangeDb {
    /// Open (or create) the database in the workspace's `.sparkle-space/` directory.
    pub fn open(workspace_path: &Path) -> rusqlite::Result<Self> {
        let sparkle_space = get_sparkle_space_dir(workspace_path);
        std::fs::create_dir_all(&sparkle_space).ok();

        let conn = Connection::open(sparkle_space.join("sparkle.db"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;

             CREATE TABLE IF NOT EXISTS sessions (
                 session_id TEXT PRIMARY KEY,
                 workspace TEXT NOT NULL,
                 started_at TEXT NOT NULL,
                 exchange_count INTEGER DEFAULT 0
             );

             CREATE TABLE IF NOT EXISTS exchanges (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 session_id TEXT NOT NULL REFERENCES sessions(session_id),
                 exchange_num INTEGER NOT NULL,
                 timestamp TEXT NOT NULL,
                 role TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
                 content TEXT NOT NULL,
                 checkpointed INTEGER NOT NULL DEFAULT 0
             );

             CREATE INDEX IF NOT EXISTS idx_exchanges_session
                 ON exchanges(session_id, exchange_num);
             CREATE INDEX IF NOT EXISTS idx_exchanges_uncheckpointed
                 ON exchanges(checkpointed) WHERE checkpointed = 0;",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Record a new session and prune old ones.
    pub fn start_session(&self, session_id: &str, workspace: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("lock not poisoned");
        conn.execute(
            "INSERT OR IGNORE INTO sessions (session_id, workspace, started_at)
             VALUES (?1, ?2, ?3)",
            params![session_id, workspace, Utc::now().to_rfc3339()],
        )?;
        // Prune old sessions
        conn.execute(
            "DELETE FROM exchanges WHERE session_id IN (
                 SELECT session_id FROM sessions
                 ORDER BY started_at DESC LIMIT -1 OFFSET ?1
             )",
            params![MAX_SESSIONS_KEPT],
        )?;
        conn.execute(
            "DELETE FROM sessions WHERE session_id NOT IN (
                 SELECT session_id FROM sessions
                 ORDER BY started_at DESC LIMIT ?1
             )",
            params![MAX_SESSIONS_KEPT],
        )?;
        Ok(())
    }

    /// Log an exchange (user prompt or assistant response).
    pub fn log_exchange(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("lock not poisoned");
        let exchange_num: i64 = conn.query_row(
            "SELECT COALESCE(MAX(exchange_num), 0) + 1 FROM exchanges WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO exchanges (session_id, exchange_num, timestamp, role, content)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, exchange_num, Utc::now().to_rfc3339(), role, content],
        )?;
        conn.execute(
            "UPDATE sessions SET exchange_count = exchange_count + 1 WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    /// Get un-checkpointed exchanges, ordered chronologically.
    pub fn get_uncheckpointed_exchanges(&self) -> rusqlite::Result<Vec<Exchange>> {
        let conn = self.conn.lock().expect("lock not poisoned");
        let mut stmt = conn.prepare(
            "SELECT timestamp, role, content
             FROM exchanges WHERE checkpointed = 0
             ORDER BY timestamp ASC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Exchange {
                    timestamp: row.get(0)?,
                    role: row.get(1)?,
                    content: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Mark all exchanges as checkpointed.
    pub fn mark_all_checkpointed(&self) -> rusqlite::Result<usize> {
        let conn = self.conn.lock().expect("lock not poisoned");
        conn.execute("UPDATE exchanges SET checkpointed = 1 WHERE checkpointed = 0", [])
    }
}

/// A single exchange record.
pub struct Exchange {
    pub timestamp: String,
    pub role: String,
    pub content: String,
}
