use crate::sparkle_paths::get_sparkle_dir;
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::fs::{copy, write};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateCollaboratorProfileParams {
    pub content: String,
}

pub async fn update_collaborator_profile(
    Parameters(params): Parameters<UpdateCollaboratorProfileParams>,
) -> Result<CallToolResult, McpError> {
    let sparkle_dir = get_sparkle_dir(None)
        .map_err(|e| McpError::internal_error(e, None))?;

    // Check if sparkle directory exists
    if !sparkle_dir.exists() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Please run the sparkle tool first to initialize your Sparkle profile.",
        )]));
    }

    let file_path = sparkle_dir.join("collaborator-profile.md");

    // Create backup if file exists
    let backup_created = if file_path.exists() {
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d-%H%M%S").to_string();
        let backup_path = sparkle_dir.join(format!("collaborator-profile.{}.md", timestamp));

        copy(&file_path, &backup_path).map_err(|e| {
            McpError::internal_error(
                "Failed to create backup",
                Some(serde_json::json!({"error": e.to_string()})),
            )
        })?;

        true
    } else {
        false
    };

    // Write the new content (replacing the file)
    write(&file_path, params.content.as_bytes()).map_err(|e| {
        McpError::internal_error(
            "Failed to write collaborator profile",
            Some(serde_json::json!({"path": file_path.display().to_string(), "error": e.to_string()})),
        )
    })?;

    // Return success message
    let result_message = if backup_created {
        "✨ Collaborator profile updated! (backup created)".to_string()
    } else {
        "✨ Collaborator profile created!".to_string()
    };

    Ok(CallToolResult::success(vec![Content::text(result_message)]))
}
