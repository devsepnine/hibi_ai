---
description: Fix a bug scientifically. Reproduce it on the reported surface, bisect the cause with runtime evidence, ship the smallest fix the evidence justifies, and prove it on the same surface.
argument-hint: "[bug report | issue | failing command]"
allowed-tools: Agent, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# Bug Fix

Every shipped line traces to runtime evidence. A change that "might help" is a hypothesis, not a fix, and it does not ship. When evidence refutes a hypothesis, revert what it motivated. `$ARGUMENTS` is the report; with none, ask for the symptom and where it was seen.

## Steps

1. **Reproduce it yourself, on the surface where it was reported.** Run the real command, drive the UI with the `run` skill or browser automation, or replay the request. Ask the user to reproduce only with a specific reason this session cannot reach the target, and only after driving it as far as it goes. If it will not fire, synthesize the trigger, tighten the conditions, or add instrumentation until it does.
2. **Bisect the cause.** List the candidate hypotheses, then eliminate them. Each pass, take the split that rules out the most remaining space and get runtime evidence for it: a log line, a probe, a debugger stop, or `git bisect` for a regression. When program state is unclear, instrument and read it as the code runs. Do not guess. After two failed fixes that share an assumption, stop and test the assumption.
3. **Confirm the mechanism** with runtime evidence before writing the fix. Remove any instrumentation that is not part of the fix.
4. **Plan the fix.** If it crosses a function boundary, run `architect` first. Write the failing test first per `tdd-workflow` when the bug has a cheap local test path. Otherwise keep the closest executable repro, within the tier waiver that skill defines.
5. **Make the smallest change the evidence justifies.** A guard that silences the crash is a symptom fix, not this.
6. **Verify on the same surface.** The original repro now passes. `INCONCLUSIVE`, or a pass on a different surface, is not a pass; see `verification-loop` Phase 7. Unit tests show branch behavior, not that the bug is gone.
7. **Check for the same class of bug elsewhere**, and run `/blast-radius` when the fix touches shared code.
8. **Keep the repro and the fix separable**, so the failing test can land before the fix in history. Commit only when the user asks.
9. **Run the post-work `code-reviewer` gate** from `CLAUDE.md` before reporting, at the rigor the change's tier sets.

## Reply

What was broken, the root cause, the fix, and how you verified it. Paste the failing-before and passing-after output verbatim, and label each claim measured, inferred, or guess.
