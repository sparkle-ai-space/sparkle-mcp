# Sparkle Paths - Implementation Spec

**Status**: Ready to implement  
**Date**: 2026-02-14  
**Blocks**: test-isolation-spec (needs centralized path handling first)

## Problem

`home_dir.join(SPARKLE_DIR)` is computed in 8 files, 9 places. No way to override for testing.

## Solution

Create `src/sparkle_paths.rs` with centralized path function. Replace all scattered computations.

## Phase 1: Create sparkle_paths.rs with tests

**Goal:** Single source of truth for sparkle directory path.

**Steps:**
1. Create `src/sparkle_paths.rs` with stub function and tests
2. Add `pub mod sparkle_paths;` to `src/lib.rs`
3. Run `cargo test get_sparkle_dir` - confirm tests fail
4. Implement `get_sparkle_dir` to make tests pass
5. Run `cargo test get_sparkle_dir` - confirm tests pass

🚦 **Stop**: Get review before proceeding.

**Tests:**
```rust
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
```

**Signature:**
```rust
pub fn get_sparkle_dir(override_path: Option<&PathBuf>) -> Result<PathBuf, &'static str>
```

## Phase 2: Update context_loader.rs

**Goal:** Use centralized function instead of inline computation.

**Steps:**
1. Update imports and replace `home_dir.join(SPARKLE_DIR)` calls
2. Run `cargo check` - fix any errors AND warnings
3. Run `cargo test` - confirm existing tests pass

Two occurrences to replace.

🚦 **Stop**: Get review before proceeding.

## Phase 3: Update tools and prompts

**Goal:** Replace all remaining scattered computations.

**Steps:**
1. Update each file one at a time
2. Run `cargo check` after each file - fix any errors AND warnings

Files:
- `src/tools/setup_sparkle.rs`
- `src/tools/create_sparkler.rs`
- `src/tools/rename_sparkler.rs`
- `src/tools/save_insight.rs`
- `src/tools/load_evolution.rs`
- `src/tools/update_collaborator_profile.rs`
- `src/prompts/sparkle.rs`

🚦 **Stop**: Get review before proceeding.

## Phase 4: Verify

**Steps:**
1. Run `cargo test` - all tests pass
2. Run `grep -r "join(SPARKLE_DIR)" src/` - should return nothing

🚦 **Stop**: Commit. Ready for test-isolation-spec.

## Success Criteria

- Single `get_sparkle_dir()` function used everywhere
- No direct `home_dir.join(SPARKLE_DIR)` in codebase
- All existing tests pass
- `override_path` parameter ready for test isolation
