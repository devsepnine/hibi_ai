---
description: Iteratively fix TypeScript and build errors. Parses error output, applies minimal fixes one at a time, verifies after each. Stops on regression.
allowed-tools: Bash, Read, Edit, Grep
model: haiku
effort: low
---

# Build and Fix

Incrementally fix TypeScript / build errors by dispatching the **build-error-resolver** agent.

## Invoke

Run the build (`npm run build` / `pnpm build`), then hand the error output to `build-error-resolver`. It fixes one error at a time with minimal diffs, verifying after each.

## Command-specific stop gates

- Stop if a fix introduces a new error (regression).
- Stop if the same error persists after 3 attempts.
- Stop if the user requests a pause.

Full diagnostic commands, error-pattern table, minimal-diff strategy, safety guards, and report format live in the `build-error-resolver` agent (`src/agents/build-error-resolver.md`). Follow that as the source of truth.
