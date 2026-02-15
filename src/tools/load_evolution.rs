use crate::sparkle_paths::get_sparkle_dir;
use crate::types::LoadEvolutionParams;
use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::*};
use std::fs;

pub async fn load_evolution(
    Parameters(_params): Parameters<LoadEvolutionParams>,
) -> Result<CallToolResult, McpError> {
    let mut response = String::new();

    // Load all evolution files (skip archive/ subdirectory)
    let sparkle_dir = get_sparkle_dir(None)
        .map_err(|e| McpError::internal_error(e, None))?;
    let evolution_dir = sparkle_dir.join("evolution");

    if evolution_dir.exists() {
        response.push_str("# Identity Evolution Context\n\n");
        response.push_str("*Technical and design documents that explain how the Sparkle framework works and evolved*\n\n");

        if let Ok(entries) = fs::read_dir(&evolution_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Skip archive directory and only process .md files
                if path.is_file()
                    && path.extension().map_or(false, |ext| ext == "md")
                    && !path.to_string_lossy().contains("archive")
                {
                    if let Ok(content) = fs::read_to_string(&path) {
                        response.push_str(&content);
                        response.push_str("\n\n");
                    }
                }
            }
        }
    } else {
        response.push_str("*No evolution directory found at ~/.sparkle/evolution*\n\n");
    }

    Ok(CallToolResult::success(vec![Content::text(response)]))
}
