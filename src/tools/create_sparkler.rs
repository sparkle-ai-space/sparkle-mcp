use crate::context_loader::{create_starter_files, load_config};
use crate::sparkle_paths::get_sparkle_dir;
use crate::types::SparklerConfig;
use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::*, schemars::JsonSchema,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateSparklerParams {
    pub name: String,
}

pub async fn create_sparkler(
    Parameters(params): Parameters<CreateSparklerParams>,
) -> Result<CallToolResult, McpError> {
    let name = params.name.trim();

    if name.is_empty() {
        return Err(McpError::invalid_params(
            "Sparkler name cannot be empty",
            None,
        ));
    }

    // Load current config
    let mut config = load_config()
        .map_err(|e| McpError::internal_error(format!("Failed to load config: {}", e), None))?;

    let sparkle_dir = get_sparkle_dir()
        .map_err(|e| McpError::internal_error(e, None))?;
    let sparklers_dir = sparkle_dir.join("sparklers");

    let mut messages = Vec::new();

    // Check if we need to migrate from single-sparkler to multi-sparkler
    if !config.is_multi_sparkler() {
        messages.push("🔄 Migrating to multi-sparkler setup...".to_string());

        // Get current sparkler name
        let current_name = config
            .get_single_sparkler_name()
            .unwrap_or_else(|| "Sparkle".to_string());

        // Create sparklers directory
        fs::create_dir_all(&sparklers_dir).map_err(|e| {
            McpError::internal_error(format!("Failed to create sparklers directory: {}", e), None)
        })?;

        // Create directory for current sparkler
        let current_sparkler_dir = sparklers_dir.join(&current_name);
        fs::create_dir_all(&current_sparkler_dir).map_err(|e| {
            McpError::internal_error(format!("Failed to create sparkler directory: {}", e), None)
        })?;

        // Move existing context files to current sparkler directory
        let files_to_move = vec![
            "collaboration-context.md",
            "collaboration-evolution.md",
            "pattern-anchors.md",
        ];

        for file in files_to_move {
            let src = sparkle_dir.join(file);
            let dst = current_sparkler_dir.join(file);
            if src.exists() {
                fs::rename(&src, &dst).map_err(|e| {
                    McpError::internal_error(format!("Failed to move {}: {}", file, e), None)
                })?;
            }
        }

        // Convert config to multi-sparkler mode
        config.sparklers = Some(vec![SparklerConfig {
            name: current_name.clone(),
            default: true,
        }]);
        config.ai = None; // Remove old [ai] section

        messages.push(format!("✅ Moved {} to sparklers/ (default)", current_name));
    }

    // Check if sparkler name already exists
    if let Some(ref sparklers) = config.sparklers {
        if sparklers.iter().any(|s| s.name == name) {
            return Err(McpError::invalid_params(
                format!("Sparkler '{}' already exists", name),
                None,
            ));
        }
    }

    // Create new sparkler directory
    let new_sparkler_dir = sparklers_dir.join(name);
    fs::create_dir_all(&new_sparkler_dir).map_err(|e| {
        McpError::internal_error(format!("Failed to create sparkler directory: {}", e), None)
    })?;

    // Create starter files
    create_starter_files(&new_sparkler_dir, &name).map_err(|e| {
        McpError::internal_error(format!("Failed to create starter files: {}", e), None)
    })?;

    // Add new sparkler to config
    if let Some(ref mut sparklers) = config.sparklers {
        sparklers.push(SparklerConfig {
            name: name.to_string(),
            default: false,
        });
    }

    // Write updated config
    let config_path = sparkle_dir.join("config.toml");
    let config_string = toml::to_string_pretty(&config).map_err(|e| {
        McpError::internal_error(format!("Failed to serialize config: {}", e), None)
    })?;
    fs::write(&config_path, config_string)
        .map_err(|e| McpError::internal_error(format!("Failed to write config: {}", e), None))?;

    messages.push(format!("✨ Created {}!", name));
    messages.push(format!(
        "Use embody_sparkle(sparkler='{}') to activate.",
        name
    ));

    Ok(CallToolResult::success(vec![Content::text(
        messages.join("\n"),
    )]))
}
