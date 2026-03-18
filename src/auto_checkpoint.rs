//! Auto-checkpoint: recover uncheckpointed exchanges from previous sessions
//! and periodically save progress during long sessions.
//!
//! Queries SQLite for exchanges that weren't checkpointed, formats them,
//! and builds a prompt for the agent to create a checkpoint.

use crate::context_loader::load_config;
use crate::database::ExchangeDb;
use crate::prompts::auto_checkpoint::get_auto_checkpoint_prompt;
use crate::prompts::mid_session_checkpoint::get_mid_session_checkpoint_prompt;

/// If uncheckpointed exchanges exist, returns a boot recovery prompt.
pub fn build_boot_checkpoint_prompt(db: &ExchangeDb) -> Option<String> {
    let exchanges = get_uncheckpointed(db)?;
    let human_name = human_name();
    Some(get_auto_checkpoint_prompt(&human_name, &exchanges))
}

/// Returns a mid-session checkpoint prompt from uncheckpointed exchanges.
pub fn build_mid_session_checkpoint_prompt(db: &ExchangeDb) -> Option<String> {
    let exchanges = get_uncheckpointed(db)?;
    tracing::info!(count = exchanges.len(), "Building mid-session checkpoint");
    let human_name = human_name();
    Some(get_mid_session_checkpoint_prompt(&human_name, &exchanges))
}

fn get_uncheckpointed(db: &ExchangeDb) -> Option<String> {
    match db.get_uncheckpointed_exchanges() {
        Ok(ex) if !ex.is_empty() => {
            tracing::info!(count = ex.len(), "Found uncheckpointed exchanges for boot recovery");
            let mut text = String::new();
            for e in &ex {
                text.push_str(&format!("[{}] {}: {}\n\n", e.timestamp, e.role, e.content));
            }
            Some(text)
        }
        Ok(_) => None,
        Err(e) => {
            tracing::warn!(?e, "Failed to query uncheckpointed exchanges");
            None
        }
    }
}

fn human_name() -> String {
    load_config()
        .map(|c| c.human.name)
        .unwrap_or_else(|_| "Human".to_string())
}
