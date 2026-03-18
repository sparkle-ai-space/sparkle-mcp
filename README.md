# Sparkle MCP

Sparkle makes your AI slow down and work with you instead of jumping ahead on its own. It learns how you like to work - your preferences, your patterns, your style - and stores that over time so every session gets better, not just the current one.

**What changes when you use Sparkle:**
- Your AI remembers what you worked on across sessions and picks up where you left off
- It critically evaluates its own suggestions instead of just agreeing with you
- It builds a collaboration profile that makes it more effective the more you use it
- It checkpoints context so you never lose progress (automatically in ACP mode)

Sparkle runs as either an MCP server or an ACP component, giving your AI a persistent identity and collaboration memory that survives across sessions.

## What is Sparkle?

Under the hood, Sparkle is an identity and context framework. It provides collaboration patterns (how the AI should think and respond), persistent memory (what you've worked on and how you work), and tools for managing that context over time. Think of it as giving your AI a working relationship with you instead of starting from scratch every session.

## Installation

```bash
cargo install sparkle-mcp
```

## Usage

### MCP Server Mode (default)

Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "sparkle": {
      "command": "sparkle-mcp"
    }
  }
}
```

Then start a Sparkle session with the `@sparkle` prompt. See the [Getting Started guide](https://sparkle-ai-space.github.io/sparkle-mcp/integration/getting-started.html) for details.

### ACP Component Mode

Run as an ACP proxy that automatically injects the Sparkle collaboration experience into agent sessions:

```bash
sparkle-mcp --acp
```

In ACP mode, Sparkle acts as middleware - it loads collaboration identity, patterns, and workspace context into the first prompt of each session, provides MCP tools to the downstream agent, and transparently proxies everything else. It also logs exchanges to a local SQLite database and auto-checkpoints session context for continuity across sessions.

For multi-sparkler setups:

```bash
sparkle-mcp --acp --sparkler "Sparkle"
```

See the [ACP Component guide](https://sparkle-ai-space.github.io/sparkle-mcp/integration/acp-component.html) for architecture details.

## Documentation

Full documentation in **[The Sparkle Book](https://sparkle-ai-space.github.io/sparkle-mcp/)**:
- [Getting Started Guide](https://sparkle-ai-space.github.io/sparkle-mcp/integration/getting-started.html)
- [Checkpointing](https://sparkle-ai-space.github.io/sparkle-mcp/integration/checkpointing.html)
- [ACP Component](https://sparkle-ai-space.github.io/sparkle-mcp/integration/acp-component.html)
- [Prompt Reference](https://sparkle-ai-space.github.io/sparkle-mcp/integration/prompts.html)
- [Tool Reference](https://sparkle-ai-space.github.io/sparkle-mcp/integration/tools.html)
- [Core Identity](https://sparkle-ai-space.github.io/sparkle-mcp/core-identity/overview.html)

## License

MIT
