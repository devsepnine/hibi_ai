# Strategic Compact Hook

Suggests manual compaction at logical intervals to preserve context better than auto-compact.

## Why Manual Over Auto-Compact

- **Auto-compact** happens at arbitrary points, often mid-task
- **Strategic compacting** preserves context through logical phases
- Compact after exploration, before execution
- Compact after completing a milestone, before starting next

## How It Works

Tracks tool usage count per session and suggests `/compact` when:
- **At threshold** (default 50 calls): First suggestion when transitioning phases
- **Regular intervals** (every 25 calls after threshold): Checkpoints for stale context

## Configuration

Set custom threshold via environment variable:
```bash
export COMPACT_THRESHOLD=100
```

## Hook Event

- **Event**: `PreToolUse`
- **Type**: `command`
- Runs before each tool use to track and suggest compaction

## Build

```bash
./build.sh
```

Builds for macOS, Windows, and Linux.
