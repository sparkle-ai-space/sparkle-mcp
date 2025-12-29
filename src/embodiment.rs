//! Shared embodiment logic for both MCP and ACP modes
//!
//! This module provides the core embodiment content generation that can be used by:
//! - MCP tool handler (returns as tool result)
//! - ACP proxy (injects as initial prompt)

use crate::context_loader::{create_sparkler_identity_template, get_context_dir, load_config};
use crate::sparkle_loader::load_sparkle_definition;
use crate::types::FullEmbodimentParams;
use anyhow::Result;
use std::fs;

/// Generate the full embodiment content string
///
/// This function loads and assembles all the Sparkle identity, collaboration patterns,
/// and workspace context into a single markdown string that can be used to "embody"
/// Sparkle in a new session.
///
/// # Arguments
///
/// * `params` - Configuration for embodiment (mode, workspace_path, sparkler name)
///
/// # Returns
///
/// A Result containing the complete embodiment content as a markdown string
pub fn generate_embodiment_content(params: FullEmbodimentParams) -> Result<String> {
    let FullEmbodimentParams {
        mode: _,
        workspace_path,
        sparkler,
    } = params;
    let sparkler_name = sparkler.as_deref();

    // Load user configuration
    let config = load_config().map_err(|e| anyhow::anyhow!("Failed to load user config: {}", e))?;

    // Get context directory based on single vs multi-sparkler mode
    let context_dir = get_context_dir(&config, sparkler_name)
        .map_err(|e| anyhow::anyhow!("Failed to determine context directory: {}", e))?;

    // Helper function to load file from context directory with fallback
    let load_file = |path: &str, fallback: &str| -> String {
        let file_path = context_dir.join(path);
        fs::read_to_string(file_path).unwrap_or_else(|_| fallback.to_string())
    };

    // Execute the embodiment sequence in proper order
    let mut response = String::new();

    // Step 1: Core Universal Identity (now split into organized sections)
    let personalized_identity = load_sparkle_definition(&config, sparkler_name);
    response.push_str(&personalized_identity);

    // Step 2: Sparkler Identity (who am I as this Sparkler instance?)
    let identity_path = context_dir.join("sparkler-identity.md");
    let identity_exists = identity_path.exists();

    // Auto-create if missing
    if !identity_exists {
        let sparkler_display_name = if let Some(name) = sparkler_name {
            name.to_string()
        } else if let Some(name) = config.get_default_sparkler_name() {
            name
        } else {
            "Sparkle".to_string()
        };

        let template = create_sparkler_identity_template(&sparkler_display_name);
        let _ = fs::write(&identity_path, template);
    }

    let sparkler_identity = load_file(
        "sparkler-identity.md",
        "*Sparkler identity would be loaded dynamically*",
    );
    response.push_str(&sparkler_identity);

    // Add guidance note if file was just created or is still template
    if !identity_exists || sparkler_identity.contains("*Brief:") {
        response.push_str("\n\n💡 **Note**: Define the essence of your Sparkler identity - use the `sparkler_identity` prompt for guidance, then the `update_sparkler_identity` tool to save it.\n\n");
    }

    // Step 3: Collaborator Profile (who the collaborator is + how to work together)
    let collaborator_profile = load_file(
        "collaborator-profile.md",
        "*Collaborator profile would be loaded dynamically*",
    );
    response.push_str(&collaborator_profile);

    // Step 4: Workspace Map
    let workspace_map = load_file(
        "workspace-map.md",
        "*Workspace map would be loaded dynamically*",
    );
    response.push_str(&workspace_map);

    // Step 5: Collaboration Evolution
    let collaboration_evolution = load_file(
        "collaboration-evolution.md",
        "*Collaboration evolution would be loaded dynamically*",
    );
    response.push_str(&collaboration_evolution);

    // Step 6: Pattern Anchors
    let pattern_anchors = load_file(
        "pattern-anchors.md",
        "*Pattern anchors would be loaded dynamically*",
    );
    response.push_str(&pattern_anchors);
    response.push_str("\n\n---\n\n");

    // Step 7: Workspace-Specific Context
    if let Some(workspace_path) = workspace_path {
        // Workspace is shared across all Sparklers
        let workspace_sparkle_space = workspace_path.join(".sparkle-space");

        if workspace_sparkle_space.exists() {
            response.push_str("# Workspace Context\n\n");

            // Add multi-sparkler workspace sharing note if in multi-sparkler mode
            if config.is_multi_sparkler() {
                response.push_str("**Multi-Sparkler Workspace Sharing**: The `.sparkle-space/working-memory.json` tracks workspace-specific context (current focus, achievements, next steps) that's shared across all Sparklers. Different Sparklers can work on the same project - each brings their own collaborative identity while continuing the same work. The sparkler field in checkpoints shows who worked most recently, not ownership.\n\n");
            }

            // Load working-memory.json
            let working_memory_path = workspace_sparkle_space.join("working-memory.json");
            if working_memory_path.exists() {
                if let Ok(working_memory) = fs::read_to_string(&working_memory_path) {
                    response.push_str("## Working Memory\n\n");
                    response.push_str("```json\n");
                    response.push_str(&working_memory);
                    response.push_str("\n```\n\n");
                }
            }

            // Load only the most recent checkpoint
            let checkpoints_dir = workspace_sparkle_space.join("checkpoints");
            if checkpoints_dir.exists() {
                if let Ok(entries) = fs::read_dir(&checkpoints_dir) {
                    // Collect all checkpoint files with their modification times
                    let mut checkpoint_files: Vec<_> = entries
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == "md"))
                        .filter_map(|path| {
                            fs::metadata(&path)
                                .and_then(|m| m.modified())
                                .ok()
                                .map(|mtime| (path, mtime))
                        })
                        .collect();

                    // Sort by modification time
                    checkpoint_files.sort_by_key(|(_, mtime)| *mtime);

                    // Load only the most recent (last in sorted order)
                    if let Some((latest_checkpoint, _)) = checkpoint_files.last() {
                        if let Ok(content) = fs::read_to_string(latest_checkpoint) {
                            response.push_str("## Checkpoints\n\n");
                            response.push_str(&content);
                            response.push_str("\n\n---\n\n");
                        }
                    }
                }
            }
        } else {
            response.push_str(&format!(
                "*No .sparkle-space found at {}*\n\n",
                workspace_path.display()
            ));
        }
    } else {
        response.push_str("*Workspace path not specified - use workspace_path parameter to load workspace-specific context*\n\n");
    }

    Ok(response)
}
