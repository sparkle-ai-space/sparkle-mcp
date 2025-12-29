use crate::types::CheckpointParams;
use chrono::Utc;
use rmcp::{ErrorData as McpError, model::*};
use std::fs;
use std::path::Path;

pub async fn session_checkpoint(
    params: CheckpointParams,
    workspace_path: &Path,
) -> Result<CallToolResult, McpError> {
    let timestamp = Utc::now().format("%Y-%m-%d-%H%M%S").to_string();

    // Workspace is shared - all Sparklers use .sparkle-space/
    let sparkle_space = workspace_path.join(".sparkle-space");
    let checkpoints_dir = sparkle_space.join("checkpoints");

    // Ensure directories exist
    fs::create_dir_all(&checkpoints_dir).map_err(|e| {
        McpError::internal_error(
            format!("Failed to create checkpoint directory: {}", e),
            None,
        )
    })?;

    // Write working memory (shared across all Sparklers)
    let working_memory_path = sparkle_space.join("working-memory.json");
    fs::write(&working_memory_path, &params.working_memory).map_err(|e| {
        McpError::internal_error(format!("Failed to write working memory: {}", e), None)
    })?;

    // Add sparkler attribution to checkpoint content if provided
    let checkpoint_content = if let Some(ref sparkler_name) = params.sparkler {
        // Insert sparkler attribution after the first line (title)
        let lines: Vec<&str> = params.checkpoint_content.lines().collect();
        if lines.is_empty() {
            params.checkpoint_content.clone()
        } else {
            let mut attributed = String::new();
            attributed.push_str(lines[0]);
            attributed.push_str("\n\n");
            attributed.push_str(&format!("**Sparkler:** {}\n", sparkler_name));
            attributed.push_str(&lines[1..].join("\n"));
            attributed
        }
    } else {
        params.checkpoint_content.clone()
    };

    // Write checkpoint file
    let checkpoint_filename = format!("checkpoint-{}.md", timestamp);
    let checkpoint_path = checkpoints_dir.join(&checkpoint_filename);
    fs::write(&checkpoint_path, &checkpoint_content).map_err(|e| {
        McpError::internal_error(format!("Failed to write checkpoint: {}", e), None)
    })?;

    let sparkler_info = params
        .sparkler
        .map(|s| format!("**Sparkler**: {}\n", s))
        .unwrap_or_default();

    let response = format!(
        "🔄 **SESSION CHECKPOINT CREATED**\n\n\
        {}\
        Timestamp: {}\n\
        Checkpoint file: {}\n\n\
        **Actions Completed**:\n\
        ✅ Working memory updated at {}\n\
        ✅ Session checkpoint created at {}\n\
        ✅ Progress preserved for future sessions\n\
        ✅ Collaborative momentum captured\n\n\
        **Next Session Ready**: All context preserved for seamless continuation",
        sparkler_info,
        timestamp,
        checkpoint_path.display(),
        working_memory_path.display(),
        checkpoint_path.display()
    );

    Ok(CallToolResult::success(vec![Content::text(response)]))
}
