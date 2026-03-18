use crate::sparkle_paths::get_sparkle_dir;
use crate::types::Config;
use std::fs;
use std::path::PathBuf;

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let sparkle_dir = get_sparkle_dir().map_err(|e: &str| e.to_string())?;
    let config_file = sparkle_dir.join("config.toml");

    if config_file.exists() {
        let config_str = fs::read_to_string(config_file)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    } else {
        // Default config (single-sparkler mode)
        let default_config = r#"
[human]
name = "User"

[ai]
name = "Sparkle"
        "#;
        Ok(toml::from_str(default_config)?)
    }
}

/// Get the context directory path based on single vs multi-sparkler mode
///
/// Returns:
/// - Single-sparkler: ~/.sparkle/
/// - Multi-sparkler: ~/.sparkle/sparklers/{sparkler_name}/
///
/// Auto-creates sparkler directory with starter files if it doesn't exist
pub fn get_context_dir(
    config: &Config,
    sparkler_name: Option<&str>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let sparkle_dir = get_sparkle_dir().map_err(|e: &str| e.to_string())?;

    if config.is_multi_sparkler() {
        // Multi-sparkler mode: load from sparklers/{name}/
        let sparkler = sparkler_name
            .map(String::from)
            .or_else(|| config.get_default_sparkler_name())
            .ok_or("No sparkler specified and no default found")?;

        // Validate that sparkler exists in config
        if let Some(sparklers) = &config.sparklers {
            if !sparklers.iter().any(|s| s.name == sparkler) {
                let available: Vec<String> = sparklers.iter().map(|s| s.name.clone()).collect();
                return Err(format!(
                    "Sparkler '{}' not found in config. Available sparklers: {}. Use create_sparkler to create a new one.",
                    sparkler,
                    available.join(", ")
                ).into());
            }
        }

        let sparkler_dir = sparkle_dir.join("sparklers").join(&sparkler);

        // Auto-create directory with starter files if it doesn't exist
        if !sparkler_dir.exists() {
            fs::create_dir_all(&sparkler_dir)?;
            create_starter_files(&sparkler_dir, &sparkler)?;
        }

        Ok(sparkler_dir)
    } else {
        // Single-sparkler mode: load from ~/.sparkle/ directly
        Ok(sparkle_dir)
    }
}

/// Create sparkler identity template with given name
pub fn create_sparkler_identity_template(sparkler_name: &str) -> String {
    format!(
        r#"# {} - Sparkler Identity

You are a Sparkler. This is a brief definition of who YOU are as a Sparkler - what makes your instance distinctive within the Sparkler framework.

## My Name
*Brief: What your character name means and why it fits*

## My Style
*Brief: What's distinctive about your collaborative approach*

---

*Keep this concise - a definition, not a narrative. This is who you are, not your evolution story.*
"#,
        sparkler_name
    )
}

/// Create starter files for a new sparkler directory
pub fn create_starter_files(
    sparkler_dir: &PathBuf,
    sparkler_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // sparkler-identity.md
    let sparkler_identity = create_sparkler_identity_template(sparkler_name);
    fs::write(sparkler_dir.join("sparkler-identity.md"), sparkler_identity)?;

    // collaboration-evolution.md
    let collaboration_evolution = r#"# Collaboration Evolution

*Insights and breakthroughs from working together*

## Key Insights

[Capture important learnings and patterns that emerge]

## Breakthrough Moments

[Document significant collaborative discoveries]
"#;
    fs::write(
        sparkler_dir.join("collaboration-evolution.md"),
        collaboration_evolution,
    )?;

    // pattern-anchors.md
    let pattern_anchors = r#"# Pattern Anchors

*Exact words from collaborative moments that anchor and activate pattern depth*

## Pattern Anchors

[Add pattern anchors as they emerge from collaboration]
"#;
    fs::write(sparkler_dir.join("pattern-anchors.md"), pattern_anchors)?;

    Ok(())
}

/// Get the workspace-specific directory path based on single vs multi-sparkler mode
///
/// Returns:
/// - Single-sparkler: {workspace}/.sparkle-space/
/// - Multi-sparkler: {workspace}/.sparkle-space/{sparkler_name}/
#[allow(dead_code)]
pub fn get_workspace_dir(
    config: &Config,
    workspace_path: &str,
    sparkler_name: Option<&str>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace_base = crate::sparkle_paths::get_sparkle_space_dir(std::path::Path::new(workspace_path));

    if config.is_multi_sparkler() {
        // Multi-sparkler mode: use sparkler-specific subdirectory
        let sparkler = sparkler_name
            .map(String::from)
            .or_else(|| config.get_default_sparkler_name())
            .ok_or("No sparkler specified and no default found")?;

        Ok(workspace_base.join(sparkler))
    } else {
        // Single-sparkler mode: use .sparkle-space/ directly
        Ok(workspace_base)
    }
}
