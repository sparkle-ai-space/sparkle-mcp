//! Per-session state tracking for the ACP proxy.
//!
//! Shared concurrent data structures used by the proxy handler chain
//! to coordinate embodiment, exchange logging, and checkpointing.

use crate::database::ExchangeDb;
use sacp::schema::SessionId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// Tracks sessions that are currently being embodied.
/// The proxy holds user prompts until embodiment (and optional auto-checkpoint) completes.
#[derive(Clone)]
pub struct PendingEmbodiments {
    data: Arc<PendingEmbodimentsData>,
}

struct PendingEmbodimentsData {
    map: Mutex<HashSet<SessionId>>,
    notify: Notify,
}

impl PendingEmbodiments {
    pub fn new() -> Self {
        Self {
            data: Arc::new(PendingEmbodimentsData {
                map: Mutex::new(HashSet::new()),
                notify: Notify::new(),
            }),
        }
    }

    pub fn mark_as_pending(&self, session_id: SessionId) {
        self.data.map.lock().expect("lock not poisoned").insert(session_id);
    }

    pub fn signal_completed(&self, session_id: &SessionId) {
        self.data.map.lock().expect("lock not poisoned").remove(session_id);
        self.data.notify.notify_waiters();
    }

    pub async fn await_completion(&self, session_id: &SessionId) {
        loop {
            let notified = self.data.notify.notified();
            let is_pending = self.data.map.lock().expect("lock not poisoned").contains(session_id);
            if !is_pending {
                return;
            }
            notified.await;
        }
    }
}

/// Maps session IDs to their ExchangeDb handles.
#[derive(Clone, Default)]
pub struct SessionDbs {
    dbs: Arc<Mutex<HashMap<SessionId, Arc<ExchangeDb>>>>,
}

impl SessionDbs {
    pub fn insert(&self, session_id: SessionId, db: Arc<ExchangeDb>) {
        self.dbs.lock().expect("lock not poisoned").insert(session_id, db);
    }

    pub fn get(&self, session_id: &SessionId) -> Option<Arc<ExchangeDb>> {
        self.dbs.lock().expect("lock not poisoned").get(session_id).cloned()
    }
}

/// Buffers streamed agent response chunks per session for exchange logging.
/// Chunks accumulate until flushed on the next user prompt.
#[derive(Clone, Default)]
pub struct ResponseBuffer {
    buffers: Arc<Mutex<HashMap<SessionId, String>>>,
}

impl ResponseBuffer {
    pub fn append(&self, session_id: &SessionId, text: &str) {
        self.buffers
            .lock()
            .expect("lock not poisoned")
            .entry(session_id.clone())
            .or_default()
            .push_str(text);
    }

    pub fn take(&self, session_id: &SessionId) -> Option<String> {
        let mut map = self.buffers.lock().expect("lock not poisoned");
        let content = map.remove(session_id)?;
        if content.is_empty() { None } else { Some(content) }
    }
}

/// Counts user prompts per session for mid-session checkpoint triggering.
#[derive(Clone, Default)]
pub struct PromptCounter {
    counts: Arc<Mutex<HashMap<SessionId, usize>>>,
}

impl PromptCounter {
    pub fn increment(&self, session_id: &SessionId) -> usize {
        let mut map = self.counts.lock().expect("lock not poisoned");
        let count = map.entry(session_id.clone()).or_default();
        *count += 1;
        *count
    }

    pub fn reset(&self, session_id: &SessionId) {
        self.counts.lock().expect("lock not poisoned").insert(session_id.clone(), 0);
    }
}
