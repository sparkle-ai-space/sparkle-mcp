# Checkpointing

## The Problem

Every AI session starts from zero. Your AI assistant has no memory of what you worked on yesterday, what decisions were made, or what's in progress. Some tools mitigate this by saving recent messages or summarizing context - but raw message history is just data. It tells the next session *what happened*, not *what it means*.

## What Checkpointing Does

A Sparkle checkpoint is a letter from one AI incarnation to its next self. Instead of replaying conversation history, the AI reasons about the session - what was accomplished, what's in progress, what decisions were made and why, what to do next - and writes a structured handoff.

The difference is like handing someone a meeting transcript vs a briefing note written by someone who was in the meeting and knows what matters.

### What a Checkpoint Contains

A checkpoint has two parts:

- **Working memory** (`working-memory.json`) - structured JSON with current focus, recent achievements, next steps, collaboration state, and key insights. This is what the next session loads to understand *where we are*.
- **Checkpoint narrative** - a markdown file summarizing progress, decisions, and context. This is the reasoning - *why* we're here and *what matters*.

Together they give the next incarnation enough understanding to continue the work, not just enough data to guess at it.

## How to Use It

### Manual Checkpoint

Say "checkpoint" during a session. Sparkle will:

1. Review what's been accomplished
2. Identify any insights worth capturing
3. Update working memory with current state
4. Write a checkpoint narrative

This is useful at natural stopping points - end of a work session, before switching tasks, or when you've reached a milestone.

### Auto-Checkpoint (ACP Mode)

When running as an [ACP component](./acp-component.md), Sparkle can checkpoint automatically because it observes the full conversation flow:

- **Boot checkpoint** - When a new session starts and there are unprocessed exchanges from a previous session, Sparkle asks the AI to checkpoint that context before your first message. This catches sessions that ended without a manual checkpoint.
- **Mid-session checkpoint** - After a set number of messages, Sparkle triggers a checkpoint to preserve progress in long sessions. This protects against losing context if a session drops unexpectedly.

Auto-checkpoints use the same mechanism as manual ones - the AI reasons about the exchanges and writes a structured handoff. The only difference is the trigger.

## Where Checkpoints Live

```
<workspace>/.sparkle-space/
├── working-memory.json          # Current state (overwritten each checkpoint)
└── checkpoints/
    ├── checkpoint-2026-03-17-200000.md
    ├── checkpoint-2026-03-18-031730.md
    └── ...                      # Accumulate over time
```

Working memory is always current - it gets overwritten on each checkpoint. Checkpoint narratives accumulate as a history that can be reviewed or cleaned up as needed.

## Is It Worth the Overhead?

If your sessions are short and self-contained, you probably don't need checkpointing. But if you:

- Work on the same project across multiple sessions
- Have long sessions where losing context would be painful
- Want the AI to understand *why* decisions were made, not just *what* was decided
- Collaborate with an AI that builds on previous work rather than starting fresh each time

Then checkpointing is what makes that continuity possible. The overhead is small - a few seconds to write a handoff - and the payoff is an AI that picks up where you left off with understanding, not just history.
