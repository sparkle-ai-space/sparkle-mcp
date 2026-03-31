# Sparkle Modes

Sparkle defaults to **full** mode, which persists workspace state (working memory, checkpoints, exchange logs) in a `.sparkle-space/` directory in your working directory.

**Core** mode disables workspace persistence entirely. Your Sparkler identity and collaboration profile (`~/.sparkle/`) still work — you just don't get `.sparkle-space/`, checkpoints, or exchange logging.

This is useful when you manage persistence through other tools, or when writing state to your working directory isn't desirable.

## Usage

### MCP

```json
{
  "mcpServers": {
    "sparkle": {
      "command": "sparkle-mcp",
      "args": ["--mode", "core"]
    }
  }
}
```

### ACP

```bash
sparkle-mcp --acp --mode core
```

The default is `full` if `--mode` is not specified.
