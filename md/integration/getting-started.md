# Getting Started

## Prerequisites

- **Rust** (1.70 or later) - [Install from rustup.rs](https://rustup.rs/)
- An MCP-compatible client or an ACP-compatible agent

## Installation

```bash
cargo install sparkle-mcp
```

This installs `sparkle-mcp` to `~/.cargo/bin/`, which should already be in your PATH if you installed Rust via rustup.

## Setup for MCP

Add Sparkle to your MCP client configuration:

```json
{
  "mcpServers": {
    "sparkle": {
      "command": "sparkle-mcp"
    }
  }
}
```

Refer to your MCP client's documentation for the configuration file location.

To start a Sparkle session, use the `@sparkle` prompt. This loads your collaboration identity, patterns, and workspace context.

## Setup for ACP

Run Sparkle as an ACP component:

```bash
sparkle-mcp --acp
```

In ACP mode, Sparkle automatically loads your collaboration identity into the first prompt of each session. No manual activation needed. See the [ACP Component guide](./acp-component.md) for details.

## First-Time Setup

Start your first Sparkle session with the `@sparkle` prompt. Sparkle will ask for your name, create your `~/.sparkle/` profile directory, and introduce itself. From then on, every session starts the same way - use `@sparkle` and Sparkle picks up where you left off.

After your first session, you can enrich your collaborator profile to help Sparkle understand how you work:

- Edit `~/.sparkle/collaborator-profile.md` to add your working style, technical expertise, and collaboration preferences
- Use the `fetch_profile_data` tool to pull information from your GitHub profile, blog, or website

## Next Steps

- **[Checkpointing](./checkpointing.md)** - How Sparkle maintains context across sessions
- **[Tool Reference](./tools.md)** - All available Sparkle tools
- **[ACP Component](./acp-component.md)** - Running Sparkle in agent composition chains
- **[Core Identity](../core-identity/overview.md)** - What makes Sparkle different

## Troubleshooting

### Server Not Found

If your client can't find Sparkle:
- Verify `~/.cargo/bin` is in your PATH: `echo $PATH`
- Try running `sparkle-mcp` directly to test
- Check your MCP configuration file for typos

### Profile Issues

If Sparkle can't load your profile:
- Verify `~/.sparkle/` directory exists
- Check that `collaborator-profile.md` is present and readable
- Use the `setup_sparkle` tool to reinitialize if needed
