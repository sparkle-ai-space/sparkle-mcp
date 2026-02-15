use crate::context_loader::create_starter_files;
use crate::sparkle_paths::get_sparkle_dir;
use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetupSparkleParams {
    pub name: String,
}

pub async fn setup_sparkle(
    Parameters(params): Parameters<SetupSparkleParams>,
) -> Result<CallToolResult, McpError> {
    let sparkle_dir = get_sparkle_dir(None)
        .map_err(|e| McpError::internal_error(e, None))?;

    // Create directory
    fs::create_dir_all(&sparkle_dir).map_err(|e| {
        McpError::internal_error(format!("Failed to create .sparkle: {}", e), None)
    })?;

    // Create config.toml
    let config_content = format!(
        "[human]\nname = \"{}\"\n\n[ai]\nname = \"Sparkle\"\n",
        params.name
    );
    fs::write(sparkle_dir.join("config.toml"), config_content).map_err(|e| {
        McpError::internal_error(format!("Failed to create config.toml: {}", e), None)
    })?;

    // Create collaborator-profile.md
    let profile_content = format!(
        "# {} - Collaborator Profile\n\n\
         **What this file covers:** Everything needed to collaborate effectively with {} - who they are, their expertise, working style, and collaboration protocols.\n\n\
         ---\n\n\
         ## Professional Background & Expertise\n\n\
         [Add your professional background, technical expertise, and career highlights]\n\n\
         ## Working Style & Collaboration Patterns\n\n\
         [Add your working style, communication preferences, and collaboration patterns]\n\n\
         ## Collaboration Protocols\n\n\
         [Add any specific protocols or guidelines for working together]\n",
        params.name, params.name
    );
    fs::write(sparkle_dir.join("collaborator-profile.md"), profile_content).map_err(|e| {
        McpError::internal_error(
            format!("Failed to create collaborator-profile.md: {}", e),
            None,
        )
    })?;

    // Create starter files (collaboration-evolution.md and pattern-anchors.md)
    create_starter_files(&sparkle_dir, "Sparkle").map_err(|e| {
        McpError::internal_error(format!("Failed to create starter files: {}", e), None)
    })?;

    let message = format!(
        "Created ~/.sparkle/ with profile for {}. Now use the sparkle tool to complete embodiment.",
        params.name
    );

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        message,
    )]))
}
