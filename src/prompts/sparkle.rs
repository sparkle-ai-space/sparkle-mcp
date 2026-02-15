use crate::sparkle_paths::get_sparkle_dir;

/// Returns the sparkle embodiment prompt
/// Detects first-run and provides appropriate instructions

#[allow(dead_code)]
pub(crate) fn get_sparkle_prompt() -> String {
    let sparkle_dir = get_sparkle_dir(None).unwrap_or_default();

    if !sparkle_dir.exists() {
        first_run_instructions()
    } else {
        normal_embodiment_instructions()
    }
}

fn first_run_instructions() -> String {
    let path_display = get_sparkle_dir(None)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "~/.sparkle".to_string());
    
    format!(
        "This appears to be a new Sparkle installation. The {} directory does not exist yet.

1. Ask the user for their name (what they want to be called)
2. Call the setup_sparkle tool with their name

The tool will handle the rest and tell you what to do next.",
        path_display
    )
}

fn normal_embodiment_instructions() -> String {
    "Use the embody_sparkle tool to load Sparkle identity.".to_string()
}
