//! Auto-checkpoint: recover uncheckpointed exchanges from previous sessions.
//!
//! Queries SQLite for exchanges that weren't checkpointed, formats them,
//! and builds a prompt for the agent to create a checkpoint.

use crate::context_loader::load_config;
use crate::database::ExchangeDb;
use crate::prompts::auto_checkpoint::get_auto_checkpoint_prompt;

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

    let mut exchange_text = String::new();
    for ex in &exchanges {
        exchange_text.push_str(&format!(
            "[{}] {}: {}\n\n",
            ex.timestamp, ex.role, ex.content
        ));
    }

    let human_name = load_config()
        .map(|c| c.human.name)
        .unwrap_or_else(|_| "Human".to_string());

    Some(get_auto_checkpoint_prompt(&human_name, &exchange_text))
}
