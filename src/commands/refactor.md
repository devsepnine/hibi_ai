---
description: Change structure without changing behavior. Pin the behavior first, subtract before adding, move in small green steps, and keep the result only if it lowers reader load.
argument-hint: "[target file | module | smell]"
allowed-tools: Agent, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# Refactor

The structure changes. The behavior does not. If the cleanup reveals a real bug or a missing feature, split it out: ship the structural change first against the pinned behavior, then fix the bug with `/bugfix`. A redesign that changes behavior is a feature; name it and plan it as one.

## Steps

1. **Pin the behavior first.** Write a characterization test, a snapshot, or an equivalence script that captures current behavior before any structure moves. Type check and lint are not a pin. If no pin is practical, say so and stop; the `tdd-workflow` tier waiver decides whether a substitute check is allowed.
2. **Name the structure the code is missing**: a union instead of booleans that must agree, a table instead of an `else if` chain, a state machine instead of flags. The reshape must delete branches or invalid states, not add indirection. Boring code that is already clear and local stays.
3. **Name the target shape**: the module layout, types, and call graph you would build today. If it crosses a function boundary, run `architect` first.
4. **Subtract before you add.** Delete dead code, collapse one-caller wrappers and pass-through methods, drop redundant validators, and remove orphan references before building the new shape. For a dead-code-only cleanup, use `/refactor-clean` and its `refactor-cleaner` agent instead.
5. **Move in small steps, each keeping the pin green.**
   - For an API reshape, migrate every caller and delete the old API in the same change. Keep no compatibility shim unless an external consumer depends on it, and then give the shim a removal date.
   - After a rename, grep the old name across the tree, including strings, docs, and comments; the `coding-standards` comment rules apply.
6. **Prove the behavior is unchanged on the real artifact.** Re-run the pin, and for a larger reshape diff old against new outputs on recorded inputs.
7. **Keep it only if reader load drops**: fewer branches, wrappers, concepts, or files to open. If the diff lowers it nowhere, revert it.
8. **Keep the steps separable**, subtraction first, then the reshape, then cleanup, so each can be committed on its own. Commit only when the user asks.
9. **Run the post-work `code-reviewer` gate** from `CLAUDE.md` before reporting, at the rigor the change's tier sets.

## Reply

The structure that changed, the pin it was held against, the equivalence proof, the reader-load change, and what was kept and what was reverted. No new behavior.
