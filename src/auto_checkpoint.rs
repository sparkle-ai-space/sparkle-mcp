//! Auto-checkpoint: recover uncheckpointed exchanges from previous sessions.
//!
//! Queries SQLite for exchanges that weren't checkpointed, formats them,
//! and builds a prompt for the agent to create a checkpoint.

use crate::context_loader::load_config;
use crate::database::ExchangeDb;
use crate::prompts::auto_checkpoint::get_auto_checkpoint_prompt;
use crate::prompts::mid_session_checkpoint::get_mid_session_checkpoint_prompt;

/// If uncheckpointed exchanges exist, returns the prompt string to send to the agent.
pub fn build_auto_checkpoint_prompt(db: &ExchangeDb) -> Option<String> {
    let exchanges = match db.get_uncheckpointed_exchanges() {
        Ok(ex) if !ex.is_empty() => ex,
        Ok(_) => return None,
        Err(e) => {
            tracing::warn!(?e, "Failed to query uncheckpointed exchanges");
            return None;
        }
    };

    tracing::info!(count = exchanges.len(), "Found uncheckpointed exchanges");

    let exchange_text = format_exchanges(&exchanges);

    let human_name = load_config()
        .map(|c| c.human.name)
        .unwrap_or_else(|_| "Human".to_string());

    Some(get_auto_checkpoint_prompt(&human_name, &exchange_text))
}

/// If uncheckpointed exchange count exceeds threshold, returns a mid-session checkpoint prompt.
pub fn build_mid_session_checkpoint_prompt(db: &ExchangeDb, threshold: usize) -> Option<String> {
    let exchanges = match db.get_uncheckpointed_exchanges() {
        Ok(ex) if ex.len() >= threshold => ex,
        Ok(_) => return None,
        Err(e) => {
            tracing::warn!(?e, "Failed to query uncheckpointed exchanges");
            return None;
        }
    };

    tracing::info!(count = exchanges.len(), "Uncheckpointed exchanges hit threshold, triggering mid-session checkpoint");

    let exchange_text = format_exchanges(&exchanges);

    let human_name = load_config()
        .map(|c| c.human.name)
        .unwrap_or_else(|_| "Human".to_string());

    Some(get_mid_session_checkpoint_prompt(&human_name, &exchange_text))
}

fn format_exchanges(exchanges: &[crate::database::Exchange]) -> String {
    let mut text = String::new();
    for ex in exchanges {
        text.push_str(&format!(
            "[{}] {}: {}\n\n",
            ex.timestamp, ex.role, ex.content
        ));
    }
    text
}
