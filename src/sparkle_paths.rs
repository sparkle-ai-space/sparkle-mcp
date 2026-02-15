//! Centralized path handling for Sparkle directories.
//!
//! Single source of truth for sparkle directory path computation.
//! Supports override for testing.

use std::path::PathBuf;

const SPARKLE_DIR: &str = ".sparkle";

/// Get the sparkle directory path.
///
/// If `override_path` is provided, returns that path directly.
/// Otherwise, returns `~/.sparkle`.
pub fn get_sparkle_dir(override_path: Option<&PathBuf>) -> Result<PathBuf, &'static str> {
    match override_path {
        Some(path) => Ok(path.clone()),
        None => {
            let home = dirs::home_dir().ok_or("Could not determine home directory")?;
            Ok(home.join(SPARKLE_DIR))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sparkle_dir_with_override() {
        let override_path = PathBuf::from("/test/path");
        let result = get_sparkle_dir(Some(&override_path)).unwrap();
        assert_eq!(result, override_path);
    }

    #[test]
    fn test_get_sparkle_dir_default() {
        let result = get_sparkle_dir(None).unwrap();
        assert!(result.ends_with(".sparkle"));
    }
}
