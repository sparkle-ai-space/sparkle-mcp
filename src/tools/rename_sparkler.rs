use crate::context_loader::load_config;
use crate::sparkle_paths::get_sparkle_dir;
use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RenameSparklerParams {
    pub new_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_name: Option<String>,
}

pub async fn rename_sparkler(
    Parameters(params): Parameters<RenameSparklerParams>,
) -> Result<CallToolResult, McpError> {
    let new_name = params.new_name.trim();

    if new_name.is_empty() {
        return Err(McpError::invalid_params(
            "Character name cannot be empty",
            None,
        ));
    }

    let mut config = load_config()
        .map_err(|e| McpError::internal_error(format!("Failed to load config: {}", e), None))?;

    let sparkle_dir = get_sparkle_dir(None)
        .map_err(|e| McpError::internal_error(e, None))?;
    let config_path = sparkle_dir.join("config.toml");

    let old_name: String;
    let mut response_parts = vec![];

    if config.is_multi_sparkler() {
        // Multi-sparkler mode
        let sparklers = config.sparklers.as_ref().unwrap();

        // Determine which sparkler to rename
        let sparkler_to_rename = if let Some(old) = &params.old_name {
            // Rename specific sparkler by name
            sparklers.iter().find(|s| &s.name == old).ok_or_else(|| {
                McpError::invalid_params(format!("Sparkler '{}' not found", old), None)
            })?
        } else {
            // Rename default sparkler
            sparklers.iter().find(|s| s.default).ok_or_else(|| {
                McpError::internal_error("No default sparkler found".to_string(), None)
            })?
        };

        old_name = sparkler_to_rename.name.clone();

        // Check if new name already exists (and it's not just renaming to itself)
        if sparklers
            .iter()
            .any(|s| s.name == new_name && s.name != old_name)
        {
            return Err(McpError::invalid_params(
                format!("A sparkler named '{}' already exists", new_name),
                None,
            ));
        }

        // Check for active working memory
        let old_dir = sparkle_dir.join("sparklers").join(&old_name);
        let working_memory_path = old_dir.join("working-memory.json");
        if working_memory_path.exists() {
            response_parts.push(format!("⚠️  Active working memory detected - consider creating a checkpoint before renaming"));
        }

        // Update sparkler name in config
        let sparklers = config.sparklers.as_mut().unwrap();
        let sparkler = sparklers.iter_mut().find(|s| s.name == old_name).unwrap();
        sparkler.name = new_name.to_string();

        // Rename directory (from current name to new name)
        let new_dir = sparkle_dir.join("sparklers").join(new_name);
        if old_dir.exists() {
            fs::rename(&old_dir, &new_dir).map_err(|e| {
                McpError::internal_error(
                    format!("Failed to rename sparkler directory: {}", e),
                    None,
                )
            })?;
            response_parts.push(format!(
                "📁 Renamed directory: sparklers/{} → sparklers/{}",
                old_name, new_name
            ));
        }
    } else {
        // Single-sparkler mode
        old_name = config
            .get_single_sparkler_name()
            .unwrap_or_else(|| "Sparkle".to_string());

        if let Some(ref mut ai) = config.ai {
            ai.name = new_name.to_string();
        }
    }

    // Write updated config
    let config_string = toml::to_string_pretty(&config).map_err(|e| {
        McpError::internal_error(format!("Failed to serialize config: {}", e), None)
    })?;

    fs::write(&config_path, config_string)
        .map_err(|e| McpError::internal_error(format!("Failed to write config: {}", e), None))?;

    let mut response = format!(
        "✨ Sparkler renamed!\n\n\
         Previous name: {}\n\
         New name: {}\n\n",
        old_name, new_name
    );

    if !response_parts.is_empty() {
        response.push_str(&response_parts.join("\n"));
        response.push_str("\n\n");
    }

    response.push_str(
        "The change will take effect on your next embodiment. \
         Your Sparkle framework and all patterns remain the same - \
         only your character name has changed.",
    );

    Ok(CallToolResult::success(vec![Content::text(response)]))
}
