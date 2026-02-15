# Spec Guide

How to write and execute specs for sparkle-mcp.

## Philosophy

Specs capture human design decisions so AI executes them faithfully. They're implementation plans, not requirements documents.

**Human owns:** Architecture, approach, judgment calls
**AI owns:** Execution of steps, catching errors, asking when unclear

**Spec creation:** Human and AI collaborate. Human has the design in mind, AI helps structure it into phases and steps. Human reviews before execution.

**Spec lifecycle:** Specs are records of how something was built, not living documents maintained in sync with code. The code is the truth. The spec is scaffolding - useful for understanding why things were done, but not updated for every subsequent change.

## TDD Rhythm

Specs use Test-Driven Development. The rhythm is:

1. **Write tests first** - Add tests to the file
2. **Watch them fail** - Run `cargo test`, confirm failure (function doesn't exist yet)
3. **Implement minimally** - Write just enough code to pass
4. **Watch them pass** - Run `cargo test`, confirm success
5. **Stop for review** - Don't proceed to next phase without confirmation

This rhythm prevents implementation rush - the urge to skip ahead because the solution seems "obvious."

## Compile Checks

Always fix both errors AND warnings. Run `cargo check` after changes - a clean build has zero warnings. Unused imports, dead code, etc. should be addressed immediately, not left as tech debt.

## Phase Structure

Each phase in a spec should have:

```markdown
### Phase N: Short description

**Goal:** What this phase accomplishes.

**Steps:**
1. Add the tests below to <file>
2. Run `cargo test` - confirm they fail
3. Implement <function> to make tests pass
4. Run `cargo test` - confirm they pass

🚦 **Stop**: Get review before proceeding to Phase N+1.

**Tests:**
\`\`\`rust
// test code
\`\`\`

**Signature:**
\`\`\`rust
// function signature
\`\`\`
```

The explicit steps make the TDD rhythm unavoidable.

## Stop Points

Every phase ends with a stop point (🚦). This creates natural review moments and prevents runaway implementation. Questions surface naturally during execution - the spec doesn't need to anticipate everything if the process catches gaps.

## Post-Execution Review

After completing all phases, do a meta-reflection:

1. **Evaluate execution against spec** - What issues came up that the spec didn't anticipate?
2. **Identify spec improvements** - Could the spec have been clearer or more complete?
3. **Update the spec** - Fix gaps so future executions go smoother
4. **Update spec-guide** - If learnings are general (not code-specific), add them here


## Rust Gotchas

Common issues that specs should anticipate:

**Dual crate roots**: If project has both `lib.rs` and `main.rs`, new modules must be declared in both with `mod module_name;`.

**Error type compatibility**: `&'static str` doesn't implement `std::error::Error`. When a function returns `Result<T, &'static str>` and caller expects `Box<dyn Error>`, use:
```rust
get_sparkle_dir(None).map_err(|e: &str| e.to_string())?
```
