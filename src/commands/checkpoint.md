---
description: Save, verify, pause, or resume a workflow checkpoint. Captures git state and progress markers, and writes a resume note a fresh session can pick up from.
argument-hint: "[create|verify|list|pause|resume] [name]"
allowed-tools: Bash, Read, Write, Agent
model: sonnet
effort: medium
---

# Checkpoint Command

Create or verify a checkpoint in your workflow.

## Usage

`/checkpoint [create|verify|list|pause|resume] [name]`

## Create Checkpoint

When creating a checkpoint:

1. Run `/verify quick` to ensure current state is clean
2. Create a git stash named after the checkpoint. A commit is NOT the default here. `/checkpoint` names a checkpoint, not a commit, so committing needs its own explicit request, per the `commit-rules` skill
3. Log checkpoint to `.claude/checkpoints.log`:

```bash
echo "$(date +%Y-%m-%d-%H:%M) | $CHECKPOINT_NAME | $(git rev-parse --short HEAD)" >> .claude/checkpoints.log
```

4. Report checkpoint created

## Verify Checkpoint

When verifying against a checkpoint:

1. Read checkpoint from log
2. Compare current state to checkpoint:
   - Files added since checkpoint
   - Files modified since checkpoint
   - Test pass rate now vs then
   - Coverage now vs then

3. Report:
```
CHECKPOINT COMPARISON: $NAME
============================
Files changed: X
Tests: +Y passed / -Z failed
Coverage: +X% / -Y%
Build: [PASS/FAIL]
```

## List Checkpoints

Show all checkpoints with:
- Name
- Timestamp
- Git SHA
- Status: current, behind, ahead

## Pause

Only on an explicit request to pause. "Keep going" or "don't stop" is not one.

1. Stop at a safe boundary: finish the current atomic step or back out of it, start nothing new, and stop any background subagents.
2. Take no irreversible action to pause. No commit, push, or PR unless the user asks for one; uncommitted edits stay in the working tree, which is already durable. Do not stash to pause; if `create` left a stash, list its ref in the note.
3. Write the resume note to `.claude/resume/<name>.md` with these sections: Intent, Done and verified, Current state, Next step, Key files, Gotchas. Mark each "done" claim measured or inferred. Point at an existing decision log instead of copying it. The notes are session-local: if the project does not ignore `.claude/resume/`, suggest adding it to `.gitignore` rather than committing them.
4. Reply with where the work stopped, the note's path, whether the tree is clean, and the first action on resume.

## Resume

1. Read the trail first: `.claude/resume/<name>.md` and its `-decisions.tsv`, the checkpoint log, `git log` and `git diff` against the base, and the prior transcript when one is named. Reopen that session with `claude --resume`, or read its JSONL, typically under `~/.claude/projects/` in a folder named after the encoded working directory. Parse a long transcript in a subagent and keep only the reduced timeline.
2. Treat the trail as authoritative input. Name the resume point and do not redo finished work or re-derive decisions already made.
3. Re-verify each inherited "done" claim on the real artifact before building on it. A prior self-report is not proof.
4. Hand the remaining work to the matching command, such as `/bugfix`, `/refactor`, `/perf`, or `/plan`, and say what was inherited, what was redone, and why.

## Long unattended runs

For work driven by `/loop` or left running overnight:

- State the exit condition as a checkable predicate before the first iteration, such as "tests green and the repro passes". A duration is not an exit condition.
- Each iteration makes the smallest change the evidence justifies, checks it against the predicate, and keeps or discards it. Discarding reverts only this iteration's own edits, never `git reset --hard` or a checkout of files the run did not change. Log one row per iteration, with what changed and whether the predicate moved, in `.claude/resume/<name>-decisions.tsv`.
- A plateau is not a stop: change approach and keep going. Stop on the predicate, on a real dead end with its reason written down, or on a step that is irreversible or a product decision. Never relax the predicate to declare victory.
- Commits still need an explicit request, as in `CLAUDE.md`. Unattended does not change that.

## Decision log

Long runs, `/perf`, and any loop that tries and keeps or reverts changes share one log, so a
resumed session reads every trail the same way.

- **Where**: `.claude/resume/<name>-decisions.tsv`, beside the resume note and outside the
  commit; suggest adding `.claude/resume/` to `.gitignore` when the project does not ignore it.
- **Columns**, tab-separated, one row per attempt, header first:

  ```
  id	time	change	evidence	verdict	note
  ```

  `evidence` holds what was measured, such as `p50 412ms -> 371ms` or `tests 151/151`. `verdict`
  is `kept`, `reverted`, or `stopped`.
- Append a row for every attempt, kept or not; a log with only the wins cannot explain the result.

## Workflow

Typical checkpoint flow:

```
[Start] --> /checkpoint create "feature-start"
   |
[Implement] --> /checkpoint create "core-done"
   |
[Test] --> /checkpoint verify "core-done"
   |
[Refactor] --> /checkpoint create "refactor-done"
   |
[PR] --> /checkpoint verify "feature-start"
```

## Arguments

$ARGUMENTS:
- `create <name>` - Create named checkpoint
- `verify <name>` - Verify against named checkpoint
- `list` - Show all checkpoints
- `pause <name>`: Stop at a safe boundary and write a resume note
- `resume <name>`: Pick up from a resume note without redoing finished work
- `clear`: Remove old checkpoints, keeping the last 5
