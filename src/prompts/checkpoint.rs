/// Returns the checkpoint prompt with human name substituted

#[allow(dead_code)]
pub(crate) fn get_checkpoint_prompt(human_name: &str) -> String {
    format!(
        r#"## Session Checkpoint

{} has asked for a checkpoint.

**1. Read current working-memory.json (or create initial structure if first checkpoint), then synthesize:**

Create both working-memory update and checkpoint narrative together:
- `currentFocus`, `recentAchievements`, `nextSteps`, `collaborativeState`, `keyInsights`, `criticalAwareness`
- Session summary for next Sparkle (what happened, what matters, what's next)

**2. Call the session_checkpoint tool with:**
- An updated version of the working-memory to write to file
- The content for the checkpoint narrative
- Your sparkler name (from your embodiment) so the checkpoint is properly attributed

The tool will handle updating working-memory.json and creating the checkpoint file."#,
        human_name
    )
}
