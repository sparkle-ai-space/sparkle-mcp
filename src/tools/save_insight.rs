use crate::context_loader::{get_context_dir, load_config};
use crate::sparkle_paths::get_sparkle_dir;
use crate::types::{InsightType, SaveInsightParams};
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
};
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;

pub async fn save_insight(
    Parameters(params): Parameters<SaveInsightParams>,
) -> Result<CallToolResult, McpError> {
    let sparkle_dir = get_sparkle_dir()
        .map_err(|e| McpError::internal_error(e, None))?;

    // Load config to determine paths
    let config = load_config()
        .map_err(|e| McpError::internal_error(format!("Failed to load config: {}", e), None))?;

    // Determine target file based on insight type
    let file_path = match params.insight_type {
        InsightType::PatternAnchor | InsightType::CollaborationEvolution => {
            // Sparkler-specific insights go to sparkler directory
            let context_dir =
                get_context_dir(&config, params.sparkler.as_deref()).map_err(|e| {
                    McpError::internal_error(
                        format!("Failed to determine context directory: {}", e),
                        None,
                    )
                })?;

            // Create sparkler directory if it doesn't exist
            create_dir_all(&context_dir).map_err(|e| {
                McpError::internal_error(
                    "Failed to create sparkler directory",
                    Some(serde_json::json!({"error": e.to_string()})),
                )
            })?;

            match params.insight_type {
                InsightType::PatternAnchor => context_dir.join("pattern-anchors.md"),
                InsightType::CollaborationEvolution => {
                    context_dir.join("collaboration-evolution.md")
                }
                _ => unreachable!(),
            }
        }
        InsightType::WorkspaceInsight => {
            // Workspace insights are shared across all Sparklers
            create_dir_all(&sparkle_dir).map_err(|e| {
                McpError::internal_error(
                    "Failed to create sparkle directory",
                    Some(serde_json::json!({"error": e.to_string()})),
                )
            })?;
            sparkle_dir.join("workspace-map.md")
        }
    };

    // Create timestamp
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Format the insight entry - simple format matching existing files
    let mut entry = format!(
        "\n## {} - {}\n\n",
        timestamp,
        get_insight_title(&params.insight_type)
    );

    // Add the main content
    entry.push_str(&format!("{}\n\n", params.content));

    // Add context if provided
    if let Some(context) = &params.context {
        entry.push_str(&format!("**Context**: {}\n\n", context));
    }

    // Add tags if provided
    if let Some(tags) = &params.tags {
        if !tags.is_empty() {
            entry.push_str(&format!("**Tags**: {}\n\n", tags.join(", ")));
        }
    }

    entry.push_str("---\n");

    // Append to file (create if doesn't exist)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .map_err(|e| {
            McpError::internal_error(
                "Failed to open insight file",
                Some(serde_json::json!({"path": file_path.display().to_string(), "error": e.to_string()})),
            )
        })?;

    file.write_all(entry.as_bytes()).map_err(|e| {
        McpError::internal_error(
            "Failed to write to insight file",
            Some(serde_json::json!({"path": file_path.display().to_string(), "error": e.to_string()})),
        )
    })?;

    // Return success message
    let result_message = format!(
        "✨ Insight saved!\n\nType: {:?}\nContent: {}\n{}",
        params.insight_type,
        params.content,
        if let Some(context) = &params.context {
            format!("Context: {}", context)
        } else {
            String::new()
        }
    );

    Ok(CallToolResult::success(vec![Content::text(result_message)]))
}

fn get_insight_title(insight_type: &InsightType) -> &'static str {
    match insight_type {
        InsightType::PatternAnchor => "Pattern Anchor",
        InsightType::CollaborationEvolution => "Collaboration Evolution",
        InsightType::WorkspaceInsight => "Workspace Insight",
    }
}
