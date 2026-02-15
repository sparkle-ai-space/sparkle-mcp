//! Centralized path handling for Sparkle directories.
//!
//! Single source of truth for sparkle directory path computation.
//! Supports SPARKLE_DIR env var override for testing.

use std::path::PathBuf;

const SPARKLE_DIR: &str = ".sparkle";

/// Get the sparkle directory path.
///
/// Checks SPARKLE_DIR env var first, otherwise returns `~/.sparkle`.
pub fn get_sparkle_dir() -> Result<PathBuf, &'static str> {
    if let Ok(path) = std::env::var("SPARKLE_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(SPARKLE_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sparkle_dir_default() {
        temp_env::with_var_unset("SPARKLE_DIR", || {
            let result = get_sparkle_dir().unwrap();
            assert!(result.ends_with(".sparkle"));
        });
    }

    #[test]
    fn test_get_sparkle_dir_from_env() {
        temp_env::with_var("SPARKLE_DIR", Some("/custom/sparkle"), || {
            let result = get_sparkle_dir().unwrap();
            assert_eq!(result, PathBuf::from("/custom/sparkle"));
        });
    }
}
