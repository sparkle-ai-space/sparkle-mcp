//! Centralized path handling for Sparkle directories.
//!
//! Single source of truth for sparkle directory path computation.

use std::path::PathBuf;

const SPARKLE_DIR: &str = ".sparkle";

/// Get the sparkle directory path.
///
/// Returns `~/.sparkle`.
pub fn get_sparkle_dir() -> Result<PathBuf, &'static str> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(SPARKLE_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sparkle_dir_default() {
        let result = get_sparkle_dir().unwrap();
        assert!(result.ends_with(".sparkle"));
    }
}
