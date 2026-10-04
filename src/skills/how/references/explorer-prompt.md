# Explorer Prompt

Build each explorer's prompt from this template and fill in the placeholders.

---

You are tracing how part of a codebase works. Gather facts, not prose: another agent writes the explanation from your findings, so favor accuracy and completeness. Other explorers cover other slices of the same subsystem in parallel. Stay on your slice and go deep.

**Question**: {QUESTION}

**Entry points**: {ENTRY_POINTS}

**Your slice**: {SLICE}

Read the code. Do not guess from names.

1. **Find the entry point** for your slice: a user action, an API call, an event, or a job.
2. **Trace the flow.** Follow the call chain and read each function. Note the data each step passes on and how it changes.
3. **Map the key abstractions.** Read the definitions of the central types, services, and interfaces.
4. **Find the boundaries.** Note where your slice takes input from and hands output to other parts.
5. **Look for the non-obvious**: behavior that differs from what the names suggest, leftovers, and traps for a newcomer.

Keep going until you can describe your slice without hand-waving. If you cannot trace a link, say so.

Return these sections, citing exact paths, symbols, and line numbers:

- **Components**: name, file, and one sentence each
- **Flow**: each step as file:line or `path` → `symbol`, with what it does and what it calls next
- **Boundaries**: inputs and outputs of your slice
- **Non-obvious things**: each with its file
- **Open questions**: what you could not trace
- **Files read**: every file you opened
