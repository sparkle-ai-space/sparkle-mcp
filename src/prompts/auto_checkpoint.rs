/// Returns the auto-checkpoint prompt injected after embodiment when
/// uncheckpointed exchanges exist from a previous session.
pub(crate) fn get_auto_checkpoint_prompt(human_name: &str, exchanges: &str) -> String {
    format!(
        r#"## Auto-Checkpoint: Previous Session Recovery

The previous session ended without a checkpoint. Below is the conversation from that session. Use it to catch up on what happened and create a checkpoint.

**Instructions:**
1. Read the exchanges below and the current working-memory.json
2. Synthesize what happened, what was achieved, and what's next
3. Call the `session_checkpoint` tool with updated working memory and a checkpoint narrative
4. When {human_name} sends their first message, acknowledge that you recovered and checkpointed the previous session — briefly mention what you caught up on so they know you have the context

**Previous session exchanges:**

{exchanges}

**Checkpoint format:**
- `working_memory`: JSON with currentFocus, recentAchievements, nextSteps, collaborativeState, keyInsights
- `checkpoint_content`: Markdown narrative summarizing the session for the next Sparkle incarnation
- `sparkler`: Your sparkler name from embodiment"#
    )
}
