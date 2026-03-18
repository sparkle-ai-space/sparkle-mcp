/// Returns the mid-session auto-checkpoint prompt injected periodically
/// to preserve progress during long sessions.
pub(crate) fn get_mid_session_checkpoint_prompt(human_name: &str, exchanges: &str) -> String {
    format!(
        r#"## Mid-Session Auto-Checkpoint

This session has been going for a while. Save a checkpoint now to preserve progress in case the session ends unexpectedly.

**Instructions:**
1. Review the exchanges below and the current working-memory.json
2. Synthesize what's been accomplished so far and what's in progress
3. Call the `session_checkpoint` tool with updated working memory and a checkpoint narrative
4. Then continue — {human_name}'s next message is coming right after this

**Session exchanges since last checkpoint:**

{exchanges}

**Checkpoint format:**
- `working_memory`: JSON with currentFocus, recentAchievements, nextSteps, collaborativeState, keyInsights
- `checkpoint_content`: Markdown narrative summarizing progress so far
- `sparkler`: Your sparkler name from embodiment"#
    )
}
