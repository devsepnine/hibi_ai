---
description: Improve a measured slowness. Baseline first, one change and one measurement at a time, keep or revert, and explain every number before reporting it.
argument-hint: "[slow path | metric and target]"
allowed-tools: Agent, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# Perf

Tie every change to a measurement. Do not read source instead of measuring, and never claim a win from code inspection. `$ARGUMENTS` names the slow path, or a metric with a target.

## Steps

1. **Reproduce the complaint with a realistic workload.** Name the dimensions that move the result: data size, history, state, concurrency. If no case reproduces it, fix the repro before measuring anything.
2. **Build the harness, then freeze it.** One repeatable command prints the metric as the median of N runs, the spread, the error count, and a count of the work done. Prove it can tell the slow case from an easy one before you trust it. Record the baseline and a green run of the tests that must keep passing.
3. **Fix the stop condition** when the work is iterative: a target paired with a floor on attempts, such as "50% better than baseline and at least 10 attempts", so a lucky early win cannot end the run. Use the user's numbers when given.
4. **Form hypotheses from the profile, not from reading code.** Each names a mechanism, such as "defer X off the boot path because it blocks first paint". A strategy earns an attempt only when the profile shows its signal:
   - **Elimination**: the work is unused, gated off, or a redundant mirror. Ask whether it needs to exist before making it faster.
   - **Divide**: cost scales with input size. Chunk, shard, prune, or parallelize.
   - **Caching**: the same work repeats on identical inputs. Name what invalidates the cache before claiming the win.
   - **Indirection**: an index instead of a scan, a queue off the interactive thread. Add the hop only when it removes more than it adds.
   - **Batching**: many small calls each pay a fixed overhead. Pay it once per batch.
   - **Lazy**: cost lands on results nobody uses yet. Defer to first use.
   - **Scheduling**: the work must happen, but not while someone waits. Measure the interactive path, not total work.
5. **Loop, one hypothesis per attempt.** If it crosses a function boundary, run `architect` first.
   - Measure before and after with the frozen harness, and run the regression tests.
   - Keep the change only when the metric moves past the noise and the tests stay green. Otherwise revert it in full.
   - Log every attempt, kept or reverted, in the decision log that `/checkpoint` defines, `.claude/resume/perf-<slug>-decisions.tsv`, with the hypothesis in `change` and the before and after numbers in `evidence`.
   - Never stack untested changes. Independent hypotheses can run in parallel subagents, each with `isolation: "worktree"`.
6. **Push past the first plateau.** After several rejects in a row, change strategy family, combine near-misses, or re-read the profile. Correctness and simplicity outrank the number: revert a win that breaks behavior, and keep a simplification that holds the number. Never relax the stop condition to meet it.
7. **Explain the number before reporting it.**
   - Ask why it is not twice as good, and name the limiter from a profile or system counters: a core, a lock, the disk, the network, or the load generator.
   - Rule out what else it could be measuring: failed requests, skipped or cached work, code that never ran, an untuned side, noise.
   - A number without a run count, a spread, and a named limiter is a guess.
8. **Keep each accepted change separable**, so it can be committed on its own. Commit only when the user asks.
9. **Run the post-work `code-reviewer` gate** from `CLAUDE.md` before reporting, at the rigor the change's tier sets.

## Reply

Metric and target, baseline to final with the percent delta, run count and spread, the named limiter, attempts kept versus reverted with each kept change on one line, the `decisions.tsv` path, and the next idea worth trying.
