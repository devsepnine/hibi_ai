---
description: Extract reusable patterns from the current session into a new skill — its description, trigger vocabulary, and progressive-disclosure body. Not for writing a note about what you learned.
allowed-tools: Read, Grep, Write
model: haiku
effort: low
---

# /learn - Extract Reusable Patterns

Analyze the current session and extract any patterns worth saving as skills.

## Trigger

Run `/learn` at any point during a session when you've solved a non-trivial problem.

## What to Extract

Look for:

1. **Error Resolution Patterns**
   - What error occurred?
   - What was the root cause?
   - What fixed it?
   - Is this reusable for similar errors?

2. **Debugging Techniques**
   - Non-obvious debugging steps
   - Tool combinations that worked
   - Diagnostic patterns

3. **Workarounds**
   - Library quirks
   - API limitations
   - Version-specific fixes

4. **Project-Specific Patterns**
   - Codebase conventions discovered
   - Architecture decisions made
   - Integration patterns

## Output Format

Write a real skill: `~/.claude/skills/<kebab-case-name>/SKILL.md`. A flat file
without frontmatter never loads, so a pattern saved that way is lost work — the
directory plus frontmatter is what makes it discoverable.

`description` is the only field that decides whether the skill ever triggers, and it
competes for a fixed budget: every installed skill's `name` + `description` has to fit in
about 8,000 characters combined (1% of the context window). Over budget, the skills that
do not fit lose their description **entirely** and collapse to a bare name — they stop
auto-triggering at all. So:

- **Target under 200 characters**, hard ceiling 220. Prose you don't write here is trigger
  reliability for every other skill.
- **`when_to_use:` saves nothing** — it is concatenated onto `description` inside the same
  budget. `keywords:` is accepted by the schema but ignored; it does nothing.
- **Keep the Korean.** A Hangul syllable costs one character (`코드리뷰` 4 vs `code review`
  11) and it is what the user actually types; a terse English-imperative description
  measurably fails Korean queries.

Four parts, in this order — the fourth only when a sibling skill is genuinely confusable:

```
<what it does: compressed noun phrase> Use when <trigger condition>. <한국어 트리거 어휘>. NOT for <confusable skill>.
```

Replace every `<...>` in the template — a placeholder left in the file loads fine and
silently never matches.

```markdown
---
name: <kebab-case-name>
description: <what it does> Use when <trigger condition>. <한국어 트리거 어휘>.
---

# <Descriptive Pattern Name>

## Problem
<the failure this prevents, specifically — what went wrong and how it looked>

## Solution
<the pattern, stated so it can be applied without re-deriving it>

## Example
<code, if it clarifies>

## When to Use
<trigger conditions, and when NOT to use it>
```

## Process

1. Review the session for extractable patterns
2. Identify the most valuable/reusable insight
3. Draft the skill file
4. Ask user to confirm before saving
5. Save to `~/.claude/skills/<name>/SKILL.md`
6. If the pattern would help users beyond this machine, run `/upstream-pr` to propose it for the distributed config — it ships as `src/skills/<name>/SKILL.md` with a `-ko.md` twin

## Notes

- Don't extract trivial fixes (typos, simple syntax errors)
- Don't extract one-time issues (specific API outages, etc.)
- Focus on patterns that will save time in future sessions
- Keep skills focused - one pattern per skill
