# Test Isolation - Implementation Spec

**Status**: Ready to implement  
**Date**: 2026-02-14  
**Depends on**: sparkle-paths-spec (complete ✅)

## Problem

`acp_embodiment_test.rs` reads real `~/.sparkle/`, causing test to fail based on user's personal config.

## Solution

Add `SPARKLE_DIR` environment variable support to `get_sparkle_dir()`. Tests set env var to fixture path.

## Phase 1: Remove parameter from get_sparkle_dir

**Goal:** Clean up unused override parameter.

**Steps:**
1. Update `get_sparkle_dir` to remove parameter
2. Update all callers to remove `None` argument
3. Remove `test_get_sparkle_dir_with_override` test
4. Update `test_get_sparkle_dir_default` to call without argument
5. Run `cargo test sparkle` - confirm tests pass
6. Run `cargo check` - fix any warnings

🚦 **Stop**: Get review before proceeding.

**New signature:**
```rust
pub fn get_sparkle_dir() -> Result<PathBuf, &'static str>
```

## Phase 2: Add env var support

**Goal:** `get_sparkle_dir()` checks `SPARKLE_DIR` env var first.

**Steps:**
1. Add `serial_test` dev dependency to `Cargo.toml`
2. Add env var check to `get_sparkle_dir`
3. Replace tests with new tests below
4. Run `cargo test sparkle` - confirm tests pass

🚦 **Stop**: Get review before proceeding.

**Implementation:**
```rust
pub fn get_sparkle_dir() -> Result<PathBuf, &'static str> {
    if let Ok(path) = std::env::var("SPARKLE_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(SPARKLE_DIR))
}
```

**Tests:**
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_get_sparkle_dir_default() {
    std::env::remove_var("SPARKLE_DIR");
    let result = get_sparkle_dir().unwrap();
    assert!(result.ends_with(".sparkle"));
}

#[test]
#[serial]
fn test_get_sparkle_dir_from_env() {
    std::env::set_var("SPARKLE_DIR", "/custom/sparkle");
    let result = get_sparkle_dir().unwrap();
    assert_eq!(result, PathBuf::from("/custom/sparkle"));
    std::env::remove_var("SPARKLE_DIR");
}
```

## Phase 3: Test fixtures

**Goal:** Create fixture files for integration test.

**Steps:**
1. Create `tests/test_assets/sparkle/config.toml`
2. Create `tests/test_assets/sparkle/sparklers/Sparkle Tester/sparkler-identity.md`
3. Verify files exist with `ls -la`

🚦 **Stop**: Get review before proceeding.

**Fixture:** `tests/test_assets/sparkle/config.toml`
```toml
[human]
name = "Tester"

[[sparklers]]
name = "Sparkle Tester"
default = true
```

**Fixture:** `tests/test_assets/sparkle/sparklers/Sparkle Tester/sparkler-identity.md`
```markdown
# Sparkle Tester - Sparkler Identity

Test fixture for integration tests.
```

## Phase 4: Update integration test

**Goal:** Test uses fixtures instead of real config.

**Steps:**
1. Add env var setup/teardown in test
2. Update expectation from "Sparkle" to "Sparkle Tester" (if name appears in output)
3. Run `cargo test acp_embodiment` - confirm passes

🚦 **Stop**: All tests pass. Commit. Then do Post-Execution Review per spec-guide.

## Success Criteria

- `cargo test` passes on any machine, regardless of `~/.sparkle/` state
- Test fixtures are checked into repo
- Production code unchanged (still defaults to `~/.sparkle/`)
- `SPARKLE_DIR` env var available for user configuration if desired
